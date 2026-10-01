# Distributed Ruby: a server answers method calls made on its objects from
# other processes over TCP. Each message is a Marshal dump preceded by its
# length, the way the drb gem writes them, and an object that cannot be
# dumped travels as a DRbObject naming the server it lives on.
require "socket"
require "monitor"

module DRb
  VERSION = "2.2.3"

  class DRbError < RuntimeError; end
  class DRbConnError < DRbError; end
  class DRbServerNotFound < DRbError; end
  class DRbBadURI < DRbError; end
  class DRbBadScheme < DRbError; end

  # An exception raised on the server that could not be dumped, carried
  # back with its class name, message, and backtrace.
  class DRbRemoteError < DRbError
    def initialize(error)
      @reason = error.class.to_s
      super("#{error.message} (#{error.class})")
      set_backtrace(error.backtrace)
    end

    attr_reader :reason
  end

  # An object that is never dumped. A server hands out a DRbObject for it
  # instead, so calls on it come back to the server.
  module DRbUndumped
    def _dump(_level)
      raise TypeError, "can't dump"
    end
  end

  # The objects this process has handed out references to, by id.
  class DRbObjectSpace
    include MonitorMixin

    def initialize
      super()
      @map = ObjectSpace::WeakMap.new
    end

    def to_id(object)
      synchronize do
        @map[object.__id__] = object
        object.__id__
      end
    end

    def to_obj(reference)
      synchronize do
        object = @map[reference]
        raise RangeError, "invalid reference" unless object.__id__ == reference
        object
      end
    end
  end

  DRB_OBJECT_SPACE = DRbObjectSpace.new

  class DRbIdConv
    def to_obj(reference)
      DRB_OBJECT_SPACE.to_obj(reference)
    end

    def to_id(object)
      object.nil? ? nil : DRB_OBJECT_SPACE.to_id(object)
    end
  end

  # Reading and writing the messages a request and its reply are made of.
  module DRbMessage
    LOAD_LIMIT = 26_214_400
    ARGUMENT_LIMIT = 256

    def self.dump(object, error = false)
      object = proxy(object, error) if object.is_a?(DRbUndumped)
      begin
        text = Marshal.dump(object)
      rescue StandardError
        text = Marshal.dump(proxy(object, error))
      end
      [text.bytesize].pack("N") + text
    end

    def self.proxy(object, error)
      error ? DRbRemoteError.new(object) : DRbObject.new(object)
    end

    def self.load(stream)
      header = stream.read(4)
      raise DRbConnError, "connection closed" if header.nil?
      raise DRbConnError, "premature header" if header.bytesize < 4
      size = header.unpack1("N")
      raise DRbConnError, "too large packet #{size}" if size > LOAD_LIMIT
      text = stream.read(size)
      raise DRbConnError, "connection closed" if text.nil?
      raise DRbConnError, "premature marshal format(can't read)" if text.bytesize < size
      Marshal.load(text)
    end

    def self.send_request(stream, reference, message, arguments, block)
      written = [dump(reference), dump(message.to_s), dump(arguments.length)]
      arguments.each { |argument| written << dump(argument) }
      written << dump(block)
      stream.write(written.join)
    end

    def self.recv_request(stream)
      reference = load(stream)
      message = load(stream)
      count = load(stream)
      raise DRbConnError, "too many arguments" if count > ARGUMENT_LIMIT
      arguments = Array.new(count) { load(stream) }
      block = load(stream)
      [reference, message, arguments, block]
    end

    def self.send_reply(stream, succeeded, result)
      stream.write(dump(succeeded) + dump(result, !succeeded))
    end

    def self.recv_reply(stream)
      [load(stream), load(stream)]
    end
  end

  # A stand-in for an object on a server, local or not. A call on it is sent
  # to that server, or made on the object itself when the server is this
  # process's.
  class DRbObject
    def self._load(dumped)
      uri, reference = Marshal.load(dumped)
      return DRb.to_obj(reference) if DRb.here?(uri)
      new_with(uri, reference)
    end

    def self.new_with(uri, reference)
      made = allocate
      made.instance_variable_set(:@uri, uri)
      made.instance_variable_set(:@ref, reference)
      made
    end

    def self.new_with_uri(uri)
      new(nil, uri)
    end

    def _dump(_level)
      Marshal.dump([@uri, @ref])
    end

    def initialize(object, uri = nil)
      @uri = nil
      @ref = nil
      if object.nil?
        @uri = uri
      else
        @uri = uri || (DRb.uri rescue nil)
        @ref = DRb.to_id(object)
      end
    end

    def __drburi
      @uri
    end

    def __drbref
      @ref
    end

    undef_method :to_s
    undef_method :to_a if method_defined?(:to_a)

    def respond_to?(name, include_private = false)
      case name
      when :_dump then true
      when :marshal_dump then false
      else method_missing(:respond_to?, name, include_private)
      end
    end

    def ==(other)
      other.is_a?(DRbObject) && @uri == other.__drburi && @ref == other.__drbref
    end
    alias eql? ==

    def hash
      [@uri, @ref].hash
    end

    def inspect
      "#<DRb::DRbObject:0x#{(__id__ * 8).to_s(16).rjust(16, "0")} @uri=#{@uri.inspect}, @ref=#{@ref.inspect}>"
    end

    def method_missing(name, *arguments, &block)
      if DRb.here?(@uri)
        object = DRb.to_obj(@ref)
        DRb.current_server.check_insecure_method(object, name)
        return object.__send__(name, *arguments, &block)
      end
      succeeded, result = DRbConn.call(@uri) do |stream|
        DRbMessage.send_request(stream, @ref, name, arguments, block)
        DRbMessage.recv_reply(stream)
      end
      return result if succeeded
      raise result if result.is_a?(Exception)
      raise DRbRemoteError, result.to_s
    end
  end

  # One connection to a server, opened for a call and closed after it.
  module DRbConn
    def self.call(uri)
      host, port = DRb.parse_uri(uri)
      stream = begin
        TCPSocket.new(host, port)
      rescue SystemCallError => error
        raise DRbConnError, "#{error.message} - #{uri}"
      end
      begin
        yield stream
      ensure
        stream.close
      end
    end
  end

  # A server: a thread accepting connections, and a thread for each one
  # answering the calls it carries until the other side closes it.
  class DRbServer
    INSECURE_METHOD = [:__send__].freeze

    attr_reader :uri, :thread, :front, :config

    def initialize(uri = nil, front = nil, config = nil)
      @config = config.is_a?(Hash) ? config.dup : {}
      @idconv = @config[:idconv] || DRbIdConv.new
      host, port = DRb.parse_uri(uri || "druby://:0")
      if host.empty?
        host = Socket.gethostname
        @listener = TCPServer.new(port)
      else
        @listener = TCPServer.new(host, port)
      end
      port = @listener.addr[1] if port == 0
      @uri = "druby://#{host}:#{port}"
      @front = front
      @sessions = []
      @thread = Thread.new { serve }
      DRb.regist_server(self)
    end

    def alive?
      @thread.alive?
    end

    def here?(uri)
      @uri == uri
    end

    def stop_service
      DRb.remove_server(self)
      @sessions.each { |session| session.kill unless session == Thread.current }
      @thread.kill unless @thread == Thread.current
      @listener.close unless @listener.closed?
    end

    def to_obj(reference)
      return front if reference.nil?
      @idconv.to_obj(reference)
    end

    def to_id(object)
      return nil if object.__id__ == front.__id__
      @idconv.to_id(object)
    end

    def check_insecure_method(object, name)
      raise ArgumentError, "#{name}:#{name.class} is not a symbol" unless name.is_a?(Symbol)
      raise SecurityError, "insecure method '#{name}'" if INSECURE_METHOD.include?(name)
      if object.private_methods.include?(name)
        raise NoMethodError, "private method '#{name}' called for #{object}:#{object.class}"
      elsif object.protected_methods.include?(name)
        raise NoMethodError, "protected method '#{name}' called for #{object}:#{object.class}"
      end
      true
    end

    private

    def serve
      loop do
        client = @listener.accept
        @sessions << Thread.new(client) { |stream| answer(stream) }
      end
    rescue IOError, SystemCallError
      nil
    end

    def answer(stream)
      Thread.current["DRb"] = { "client" => stream, "server" => self }
      loop do
        begin
          reference, message, arguments, block = DRbMessage.recv_request(stream)
        rescue DRbConnError
          break
        end
        succeeded, result = invoke(reference, message, arguments, block)
        DRbMessage.send_reply(stream, succeeded, result)
      end
    ensure
      stream.close unless stream.closed?
    end

    def invoke(reference, message, arguments, block)
      object = to_obj(reference)
      name = message.to_sym
      check_insecure_method(object, name)
      result = if block
        object.__send__(name, *arguments) { |*yielded| block.call(*yielded) }
      else
        object.__send__(name, *arguments)
      end
      [true, result]
    rescue StandardError, ScriptError => error
      [false, error]
    end
  end

  @primary_server = nil
  @servers = []
  @mutex = Thread::Mutex.new

  class << self
    attr_accessor :primary_server
    attr_reader :mutex

    def start_service(uri = nil, front = nil, config = nil)
      @primary_server = DRbServer.new(uri, front, config)
    end

    def stop_service
      @primary_server&.stop_service
      @primary_server = nil
    end

    def current_server
      held = Thread.current["DRb"]
      server = held && held["server"] || @primary_server
      raise DRbServerNotFound unless server&.alive?
      server
    end

    def uri
      current_server.uri
    end

    def front
      current_server.front
    end

    def thread
      @primary_server&.thread
    end

    def config
      current_server.config
    rescue DRbServerNotFound
      {}
    end

    def here?(uri)
      @servers.any? { |server| server.here?(uri) }
    end

    def to_obj(reference)
      current_server.to_obj(reference)
    end

    def to_id(object)
      current_server.to_id(object)
    end

    def regist_server(server)
      @mutex.synchronize { @servers << server }
    end

    def remove_server(server)
      @mutex.synchronize { @servers.delete(server) }
    end

    def fetch_server(uri)
      @servers.find { |server| server.here?(uri) }
    end

    # The host and the port a `druby://host:port` URI names.
    def parse_uri(uri)
      matched = %r{\Adruby://(.*?):(\d+)(\?(.*))?\z}.match(uri)
      raise DRbBadURI, "can't parse uri:#{uri}" unless matched
      [matched[1], matched[2].to_i]
    end
  end
end

DRbObject = DRb::DRbObject
DRbUndumped = DRb::DRbUndumped
DRbIdConv = DRb::DRbIdConv
