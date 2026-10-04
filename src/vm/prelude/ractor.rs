pub(super) const SOURCE: &str = r##"
# A Ractor runs its block on a thread of its own, with the arguments it was
# given copied in, and hands back what the block answered.
class Ractor
  class Error < RuntimeError
  end

  # What a Ractor's block raised, carried to whoever asks for its value.
  class RemoteError < Error
    attr_reader :ractor

    def initialize(message = "thrown by remote Ractor.", ractor = nil)
      super(message)
      @ractor = ractor
    end
  end

  class ClosedError < StopIteration
  end

  class IsolationError < Error
  end

  class MovedError < Error
  end

  class UnsafeError < Error
  end

  # What an object sent with `move: true` leaves behind in the Ractor that
  # sent it.
  class MovedObject < BasicObject
  end

  # Where one Ractor receives what others send it. Only the Ractor that made
  # a port receives from it, and any Ractor may send to it.
  class Port
    @__next_id = 1

    def self.__take_id__
      taken = @__next_id
      @__next_id += 1
      taken
    end

    def initialize
      @__owner = Ractor.current
      @__id = Port.__take_id__
      @__queue = Thread::Queue.new
    end

    # The port a Ractor receives on by default, which is number 0.
    def __as_default__(owner)
      @__owner = owner
      @__id = 0
      @__queue = Thread::Queue.new
      self
    end

    def send(held, move: false)
      raise ClosedError, "The port was already closed" if @__queue.closed?
      @__queue << Ractor.__carry__(held, move)
      self
    end

    def <<(held)
      send(held)
    end

    def receive
      unless Ractor.current.equal?(@__owner)
        raise Ractor::Error, "only allowed from the creator Ractor of this port"
      end
      if @__queue.closed? && @__queue.empty?
        raise ClosedError, "The port was already closed"
      end
      held = @__queue.pop
      if held.nil? && @__queue.closed? && @__queue.empty?
        raise ClosedError, "The port was already closed"
      end
      held
    end

    def close
      @__queue.close
      nil
    end

    def closed?
      @__queue.closed?
    end

    def __waiting__
      !@__queue.empty?
    end
    private :__as_default__, :__waiting__

    def inspect
      "#<Ractor::Port to:##{@__owner.__number__} id:#{@__id}>"
    end
  end

  # The number the next Ractor is given. The main Ractor is number 1.
  @__next_id = 2
  @__started = []
  # The Procs `make_shareable` has made shareable.
  @__shareable_procs = {}.compare_by_identity

  # The kinds of object that stay with the Ractor that made them.
  UNSHAREABLE_KINDS = [
    Thread, Thread::Mutex, Thread::Queue, Thread::ConditionVariable,
    Enumerator, Fiber, Binding, Random
  ]

  def self.new(*args, name: nil, &block)
    raise ArgumentError, "must be called with a block" if block.nil?
    unless name.nil? || name.is_a?(String)
      raise TypeError, "no implicit conversion of #{name.class} into String"
    end
    unless @__warned
      @__warned = true
      warn "Ractor API is experimental and may change in future versions of Ruby.",
           uplevel: 1, category: :experimental
    end
    Ractor.__refuse_outer_locals__(block)
    copied = args.map { |held| Ractor.__copy__(held) }
    # Making a Ractor takes a port number for the port its value comes
    # back through.
    Port.__take_id__
    written_at = caller_locations(1, 1).first
    made = allocate
    made.__start__(@__next_id, name, "#{written_at.path}:#{written_at.lineno}", copied, block)
    @__next_id += 1
    @__started << made
    made
  end

  # A block that reads or writes a local of the scope it was written in
  # cannot run apart from that scope.
  def self.__refuse_outer_locals__(block)
    outer = block.__outer_locals__
    return if outer.empty?
    raise ArgumentError, "can not isolate a Proc because it accesses outer variables (#{outer.join(", ")})."
  end

  def self.main
    @__main ||= allocate.__as_main__
  end

  def self.current
    held = Thread.current.__ractor__
    held.nil? ? main : held
  end

  def self.main?
    current.equal?(main)
  end

  # How many Ractors are running, the main one among them.
  def self.count
    @__started = @__started.select(&:__alive__)
    @__started.size + 1
  end

  # Whether an object can be handed to another Ractor as it is rather than
  # copied: a value that cannot change, or a frozen object whose parts are
  # all shareable.
  def self.shareable?(held)
    __shareable__(held, {})
  end

  def self.__shareable__(held, seen)
    case held
    when Integer, Float, Symbol, NilClass, TrueClass, FalseClass, Module, Ractor, Ractor::Port
      return true
    end
    return @__shareable_procs.key?(held) if held.is_a?(Proc)
    return false unless held.frozen?
    return true if seen.key?(held.__id__)
    seen[held.__id__] = true
    __parts__(held).all? { |part| __shareable__(part, seen) }
  end

  # The objects `held` refers to that another Ractor would reach through it.
  def self.__parts__(held)
    parts = held.instance_variables.map { |name| held.instance_variable_get(name) }
    case held
    when Array then parts.concat(held)
    when Hash then parts.concat(held.keys).concat(held.values).push(held.default)
    when Range then parts.push(held.begin, held.end)
    when Struct then parts.concat(held.to_a)
    end
    parts
  end

  # Freeze `held` and everything it refers to, so it can be handed to any
  # Ractor as it is. With `copy: true` a deep copy is frozen instead.
  def self.make_shareable(held, copy: false)
    held = Marshal.load(Marshal.dump(held)) if copy && !shareable?(held)
    __freeze_deep__(held, {}.compare_by_identity)
    held
  end

  def self.__freeze_deep__(held, seen)
    case held
    when Integer, Float, Symbol, NilClass, TrueClass, FalseClass, Module, Ractor
      return
    end
    return if seen.key?(held)
    seen[held] = true
    if UNSHAREABLE_KINDS.any? { |kind| held.is_a?(kind) }
      raise Ractor::Error, "can not make shareable object for #{held.inspect}"
    end
    return __make_proc_shareable__(held) if held.is_a?(Proc)
    __parts__(held).each { |part| __freeze_deep__(part, seen) }
    held.freeze
  end

  # A Proc is shareable when its `self` is, it assigns to no local of the
  # scope it was written in, and every such local it reads holds something
  # shareable.
  def self.__make_proc_shareable__(held)
    return if @__shareable_procs.key?(held)
    unless shareable?(held.binding.receiver)
      raise IsolationError, "Proc's self is not shareable: #{held.inspect}"
    end
    assigned = held.__outer_locals_assigned__
    unless assigned.empty?
      raise ArgumentError,
            "can not make a Proc shareable because it accesses outer variables (#{assigned.join(", ")})."
    end
    held.__outer_values__.each do |named, value, reassigned|
      if reassigned
        raise IsolationError,
              "cannot make a shareable Proc because the outer variable '#{named}' may be reassigned."
      end
      next if shareable?(value)
      raise IsolationError,
            "cannot make a shareable Proc because it can refer unshareable object #{value.inspect} from variable '#{named}'"
    end
    held.freeze
    @__shareable_procs[held] = true
  end

  # A shareable Proc running the block with `self` set to the value given
  # under the `self:` keyword, nil when none is.
  def self.shareable_proc(**options, &block)
    Ractor.__shareable_copy__(block, options, false)
  end

  def self.shareable_lambda(**options, &block)
    Ractor.__shareable_copy__(block, options, true)
  end

  def self.__shareable_copy__(block, options, lambda)
    raise ArgumentError, "tried to create Proc object without a block" if block.nil?
    made = block.__with_self__(options.fetch(:self, nil), lambda)
    Ractor.__make_proc_shareable__(made)
    made
  end

  # What a port hands on for `held`: the object itself when it is
  # shareable, and otherwise a deep copy. Moving it leaves the sender's
  # reference unusable.
  def self.__carry__(held, move)
    carried = __copy__(held)
    __ractor_move__(held) if move && !carried.equal?(held)
    carried
  end

  def self.[](key)
    current[key]
  end

  def self.[]=(key, value)
    current[key] = value
  end

  # What the current Ractor keeps under `key`, storing what the block
  # answers there first when it keeps nothing yet.
  def self.store_if_absent(key)
    raise LocalJumpError, "no block given" unless block_given?
    locals = current.__locals__
    named = __local_key__(key)
    return locals[named] if locals.key?(named)
    locals[named] = yield(nil)
  end

  def self.__local_key__(key)
    return key if key.is_a?(Symbol)
    return key.to_sym if key.is_a?(String)
    raise TypeError, "#{key.inspect} is not a symbol nor a string"
  end

  def self.receive
    current.default_port.receive
  end

  class << self
    alias recv receive
  end

  # The first of the ports and Ractors given to have something, answered
  # with what it had: a value sent to a port, or what a Ractor ended with.
  def self.select(*sources)
    raise ArgumentError, "specify at least one Ractor::Port or Ractor" if sources.empty?
    sources.each do |source|
      next if source.is_a?(Port) || source.is_a?(Ractor)
      raise ArgumentError, "should be Ractor::Port or Ractor"
    end
    loop do
      sources.each do |source|
        if source.is_a?(Port)
          return [source, source.receive] if source.__send__(:__waiting__)
        elsif !source.__alive__
          return [source, source.value]
        end
      end
      sleep 0.001
    end
  end

  # What another Ractor is handed for `held`: the object itself when it is
  # shareable, and a deep copy otherwise.
  def self.__copy__(held)
    return held if shareable?(held)
    begin
      Marshal.load(Marshal.dump(held))
    rescue TypeError
      raise TypeError, "allocator undefined for #{held.class}"
    end
  end

  def __start__(id, name, written_at, copied, block)
    @__id = id
    @__name = name
    @__written_at = written_at
    ractor = self
    @__thread = Thread.new do
      Thread.current.__ractor__ = ractor
      ractor.instance_exec(*copied, &block)
    end
    self
  end

  def __as_main__
    @__id = 1
    @__name = nil
    @__thread = Thread.main
    self
  end

  def __alive__
    @__thread.alive?
  end

  def __number__
    @__id
  end

  def default_port
    @__default_port ||= Port.allocate.__send__(:__as_default__, self)
  end

  def send(held, move: false)
    default_port.send(held, move: move)
    self
  end

  def <<(held)
    send(held)
  end

  def close
    default_port.close
  end

  # What this Ractor keeps under `key`, which only the Ractor itself reads.
  def [](key)
    unless equal?(Ractor.current)
      raise RuntimeError, "Cannot get ractor local storage for non-current ractor"
    end
    __locals__[Ractor.__local_key__(key)]
  end

  def []=(key, value)
    unless equal?(Ractor.current)
      raise RuntimeError, "Cannot set ractor local storage for non-current ractor"
    end
    __locals__[Ractor.__local_key__(key)] = value
  end

  def __locals__
    @__locals ||= {}
  end

  def name
    @__name
  end

  # What the block answered, once it has ended. What it raised is raised
  # here as a RemoteError whose cause it is.
  def value
    @__thread.value
  rescue Exception => trouble
    raise RemoteError.new("thrown by remote Ractor.", self), cause: trouble
  end

  def join
    value
    self
  end

  def inspect
    state = if !@__thread.alive?
      "terminated"
    elsif @__thread.status == "sleep" && !@__thread.equal?(Thread.current)
      "blocking"
    else
      "running"
    end
    named = @__name.nil? ? "" : " #{@__name}"
    written = @__written_at.nil? ? "" : " #{@__written_at}"
    "#<Ractor:##{@__id}#{named}#{written} #{state}>"
  end

  alias to_s inspect
end
"##;
