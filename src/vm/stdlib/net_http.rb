require 'uri'
require 'stringio'
require 'timeout'

module Net
  class Protocol
  end

  class ProtocolError < StandardError
  end

  class ProtoSyntaxError < ProtocolError
  end

  class ProtoFatalError < ProtocolError
  end

  class ProtoUnknownError < ProtocolError
  end

  class ProtoServerError < ProtocolError
  end

  class ProtoAuthError < ProtocolError
  end

  class ProtoCommandError < ProtocolError
  end

  class ProtoRetriableError < ProtocolError
  end

  class HTTPBadResponse < StandardError
  end

  class HTTPHeaderSyntaxError < StandardError
  end

  class OpenTimeout < Timeout::Error
  end

  class ReadTimeout < Timeout::Error
  end

  class WriteTimeout < Timeout::Error
  end

  module HTTPExceptions
    def initialize message, response = nil
      super message
      @response = response
    end

    def response
      @response
    end

    def data
      @response
    end
  end

  class HTTPError < ProtocolError
    include HTTPExceptions
  end

  class HTTPRetriableError < ProtoRetriableError
    include HTTPExceptions
  end

  class HTTPClientException < ProtoServerError
    include HTTPExceptions
  end

  HTTPServerException = HTTPClientException

  class HTTPFatalError < ProtoFatalError
    include HTTPExceptions
  end

  # Turns a written chunk into an HTTP/1.1 chunked-transfer frame.
  class ReadAdapter
    def initialize block
      @block = block
    end

    def inspect
      "#<#{self.class}>"
    end

    def << text
      @block.call text if @block
      self
    end
  end

  class BufferedIO
    def initialize io, read_timeout: 60, write_timeout: 60, continue_timeout: nil, debug_output: nil
      @io = io
      @read_timeout = read_timeout
      @write_timeout = write_timeout
      @continue_timeout = continue_timeout
      @debug_output = debug_output
    end

    attr_reader :io
    attr_accessor :read_timeout
    attr_accessor :write_timeout
    attr_accessor :continue_timeout
    attr_accessor :debug_output

    def inspect
      "#<#{self.class} io=#{@io}>"
    end

    def closed?
      @io.closed?
    end

    def close
      @io.close
    end

    def eof?
      @io.eof?
    end

    def write text
      @io.write text
    end

    def << text
      write text
      self
    end

    def readuntil terminator = "\n", ignore_eof = false
      line = @io.gets terminator
      if line.nil?
        raise EOFError, 'end of file reached' unless ignore_eof
        return ''
      end
      line
    end

    def readline
      readuntil("\n").chomp
    end

    def read length = nil, ignore_eof = false
      text = length.nil? ? @io.read : @io.read(length)
      if text.nil? || text.empty?
        raise EOFError, 'end of file reached' unless ignore_eof
        return ''
      end
      text
    end

    def read_all
      @io.read || ''
    end
  end

  module HTTPHeader
    def initialize_http_header initheader = nil
      @header = {}
      return unless initheader
      initheader.each do |key, value|
        name = key.to_s.downcase
        warn "net/http: duplicated HTTP header: #{key}" if $VERBOSE && @header.key?(name)
        if value.nil?
          warn "net/http: nil HTTP header: #{key}" if $VERBOSE
        else
          @header[name] = [value.to_s.strip]
        end
      end
    end

    def [] key
      values = @header[key.to_s.downcase]
      values && values.join(', ')
    end

    def []= key, value
      name = key.to_s.downcase
      if !value
        @header.delete name
        return value
      end
      @header[name] = value.is_a?(Array) ? value.map { |entry| entry.to_s } : [value.to_s]
      value
    end

    def add_field key, value
      name = key.to_s.downcase
      added = value.is_a?(Array) ? value.map { |entry| entry.to_s } : [value.to_s]
      if @header.key? name
        @header[name] = @header[name] + added
      else
        @header[name] = added
      end
    end

    def get_fields key
      values = @header[key.to_s.downcase]
      values && values.dup
    end

    def fetch key, *args, &block
      warn 'warning: block supersedes default value argument' if block && !args.empty?
      name = key.to_s.downcase
      return @header[name].join(', ') if @header.key? name
      return block.call name if block
      return args[0] unless args.empty?
      raise KeyError, "key not found: #{name.inspect}"
    end

    def each_header
      return to_enum(:each_header) unless block_given?
      @header.each do |name, values|
        yield name, values.join(', ')
      end
      self
    end

    def each &block
      each_header(&block)
    end

    def each_name
      return to_enum(:each_name) unless block_given?
      @header.each_key do |name|
        yield name
      end
      self
    end

    def each_key &block
      each_name(&block)
    end

    def each_value
      return to_enum(:each_value) unless block_given?
      @header.each_value do |values|
        yield values.join(', ')
      end
      self
    end

    def each_capitalized_name
      return to_enum(:each_capitalized_name) unless block_given?
      @header.each_key do |name|
        yield capitalize(name)
      end
      self
    end

    def each_capitalized
      return to_enum(:each_capitalized) unless block_given?
      @header.each do |name, values|
        yield capitalize(name), values.join(', ')
      end
      self
    end

    def canonical_each &block
      each_capitalized(&block)
    end

    def capitalize name
      name.to_s.split('-').map { |part| part.capitalize }.join('-')
    end
    private :capitalize

    def delete key
      @header.delete key.to_s.downcase
    end

    def key? key
      @header.key? key.to_s.downcase
    end

    def to_hash
      @header.dup
    end

    def size
      @header.size
    end

    def length
      @header.size
    end

    def content_length
      return nil unless @header.key? 'content-length'
      digits = self['Content-Length'].slice(/\d+/)
      raise HTTPHeaderSyntaxError, 'wrong Content-Length format' if digits.nil?
      digits.to_i
    end

    def content_length= length
      if !length
        @header.delete 'content-length'
        return nil
      end
      digits = length.to_s.slice(/\A\s*\d+/)
      @header['content-length'] = [digits.nil? ? '0' : digits.strip]
      length
    end

    def content_range
      return nil unless @header.key? 'content-range'
      matched = %r{\A\s*(\w+)\s+(\d+)-(\d+)/(\d+|\*)}.match self['Content-Range']
      raise HTTPHeaderSyntaxError, 'wrong Content-Range format' if matched.nil?
      return nil unless matched[1] == 'bytes'
      matched[2].to_i..matched[3].to_i
    end

    def range_length
      span = content_range
      span && (span.end - span.begin + 1)
    end

    def range
      return nil unless @header.key? 'range'
      value = self['Range']
      matched = /\Abytes=(.+)\z/.match value
      raise HTTPHeaderSyntaxError, "invalid syntax for byte-ranges-specifier: '#{value}'" if matched.nil?
      matched[1].split(',').map do |spec|
        parts = /\A\s*(\d+)?\s*-\s*(\d+)?\s*\z/.match spec
        raise HTTPHeaderSyntaxError, "invalid byte-range-spec: '#{spec}'" if parts.nil?
        first = parts[1]
        last = parts[2]
        if first && last
          raise HTTPHeaderSyntaxError, 'last-byte-pos MUST greater than or equal to first-byte-pos' if first.to_i > last.to_i
          first.to_i..last.to_i
        elsif first
          first.to_i..-1
        elsif last
          -last.to_i..-1
        else
          raise HTTPHeaderSyntaxError, 'range is not specified'
        end
      end
    end

    def set_range first, length = nil
      unless first
        @header.delete 'range'
        return nil
      end
      if length
        raise HTTPHeaderSyntaxError, 'invalid range' if first < 0 || length < 0
        spec = "#{first.to_i}-#{first.to_i + length.to_i - 1}"
      elsif first.is_a? Range
        first_pos = first.first
        last_pos = first.end
        last_pos -= 1 if first.exclude_end?
        if first_pos < 0 && last_pos < 0
          spec = "#{first_pos.to_i}"
        elsif first_pos < 0 || last_pos < -1
          raise HTTPHeaderSyntaxError, 'invalid range'
        elsif last_pos < 0
          spec = "#{first_pos.to_i}-"
        else
          raise HTTPHeaderSyntaxError, 'invalid range' if last_pos < first_pos
          spec = "#{first_pos.to_i}-#{last_pos.to_i}"
        end
      elsif first < 0
        spec = "#{first.to_i}"
      else
        spec = "0-#{first.to_i - 1}"
      end
      @header['range'] = ["bytes=#{spec}"]
      first
    end

    def range= first, length = nil
      set_range first, length
    end

    def chunked?
      value = self['Transfer-Encoding']
      return false unless value
      value.split(/[^\-\w]+/).any? { |token| token.downcase == 'chunked' }
    end

    def content_type
      main = main_type
      return nil unless main
      sub = sub_type
      sub ? "#{main}/#{sub}" : main
    end

    def main_type
      value = self['Content-Type']
      return nil unless value
      value.split(';').first.to_s.split('/')[0].to_s.strip
    end

    def sub_type
      value = self['Content-Type']
      return nil unless value
      parts = value.split(';').first.to_s.split('/')
      return nil if parts.size < 2
      parts[1].strip
    end

    def type_params
      value = self['Content-Type']
      return {} unless value
      params = {}
      value.split(';')[1..-1].to_a.each do |chunk|
        name, setting = chunk.split('=', 2)
        params[name.to_s.strip] = setting.to_s.strip if setting
      end
      params
    end

    def set_content_type type, params = {}
      pieces = [type]
      params.each do |name, setting|
        pieces = pieces + ["#{name}=#{setting}"]
      end
      @header['content-type'] = [pieces.join('; ')]
    end

    def content_type= type, params = {}
      set_content_type type, params
    end

    def set_form_data params, separator = '&'
      self.body = params.map do |name, setting|
        "#{URI.encode_www_form_component name.to_s}=#{URI.encode_www_form_component setting.to_s}"
      end.join(separator)
      self.content_type = 'application/x-www-form-urlencoded'
    end

    def form_data= params, separator = '&'
      set_form_data params, separator
    end

    def basic_auth account, password
      @header['authorization'] = [basic_encode(account, password)]
    end

    def proxy_basic_auth account, password
      @header['proxy-authorization'] = [basic_encode(account, password)]
    end

    def basic_encode account, password
      "Basic #{["#{account}:#{password}"].pack 'm0'}"
    end
    private :basic_encode
  end

  class HTTPGenericRequest
    include HTTPHeader

    def initialize method, request_has_body, response_has_body, uri_or_path, initheader = nil
      @method = method
      @request_has_body = request_has_body
      @response_has_body = response_has_body
      @uri = uri_or_path.is_a?(String) ? nil : uri_or_path
      @path = @uri ? @uri.request_uri : uri_or_path
      @decode_content = false
      initialize_http_header initheader
      self['Accept'] = '*/*' unless key? 'Accept'
      self['User-Agent'] = 'Ruby' unless key? 'User-Agent'
      @body = nil
      @body_stream = nil
      @body_data = nil
    end

    attr_reader :method
    attr_reader :path
    attr_reader :uri
    attr_reader :decode_content
    attr_reader :body_stream

    def inspect
      "#<#{self.class} #{@method}>"
    end

    def request_body_permitted?
      @request_has_body
    end

    def response_body_permitted?
      @response_has_body
    end

    def body_exist?
      warn 'Net::HTTPRequest#body_exist? is obsolete; use response_body_permitted?' if $VERBOSE
      @response_has_body
    end

    def body
      @body
    end

    def body= text
      @body = text
      @body_stream = nil
      text
    end

    def body_stream= stream
      @body = nil
      @body_stream = stream
      stream
    end

    def set_body_internal text
      raise ArgumentError, 'both of body argument and HTTPRequest#body set' if text && (@body || @body_stream)
      self.body = text if text
      self.body = '' if @body.nil? && @body_stream.nil? && request_body_permitted?
    end

    def exec sock, version, path
      if @body
        send_request_with_body sock, version, path, @body
      elsif @body_stream
        send_request_with_body_stream sock, version, path, @body_stream
      else
        write_header sock, version, path
      end
    end

    def write_header sock, version, path
      lines = ["#{@method} #{path} HTTP/#{version}\r\n"]
      each_capitalized do |name, value|
        lines = lines + ["#{name}: #{value}\r\n"]
      end
      sock.write "#{lines.join}\r\n"
    end
    private :write_header

    def supply_default_content_type
      return if key? 'Content-Type'
      warn 'net/http: Content-Type did not set; using application/x-www-form-urlencoded' if $VERBOSE
      set_content_type 'application/x-www-form-urlencoded'
    end
    private :supply_default_content_type

    def send_request_with_body sock, version, path, body
      self.content_length = body.bytesize
      delete 'Transfer-Encoding'
      supply_default_content_type
      write_header sock, version, path
      sock.write body
    end
    private :send_request_with_body

    def send_request_with_body_stream sock, version, path, stream
      unless content_length || chunked?
        raise ArgumentError, "Content-Length not given and Transfer-Encoding is not `chunked'"
      end
      supply_default_content_type
      write_header sock, version, path
      if chunked?
        while (chunk = stream.read 1024) && !chunk.empty?
          sock.write "#{chunk.bytesize.to_s 16}\r\n#{chunk}\r\n"
        end
        sock.write "0\r\n\r\n"
      else
        while (chunk = stream.read 1024) && !chunk.empty?
          sock.write chunk
        end
      end
    end
    private :send_request_with_body_stream
  end

  class HTTPRequest < HTTPGenericRequest
    def initialize path, initheader = nil
      super self.class::METHOD,
            self.class::REQUEST_HAS_BODY,
            self.class::RESPONSE_HAS_BODY,
            path, initheader
    end
  end

  class HTTPResponse
    def self.body_permitted?
      self::HAS_BODY
    end

    def self.exception_type
      self::EXCEPTION_TYPE
    end

    include HTTPHeader

    def initialize http_version, code, message
      @http_version = http_version
      @code = code
      @message = message
      initialize_http_header nil
      @body = nil
      @read = false
      @body_exist = false
      @socket = nil
      @decode_content = false
    end

    attr_reader :http_version
    attr_reader :code
    attr_reader :message
    attr_accessor :decode_content
    attr_accessor :uri

    def msg
      @message
    end

    def response
      self
    end

    def header
      self
    end

    def read_header
      self
    end

    def inspect
      "#<#{self.class} #{@code} #{@message} readbody=#{@read}>"
    end

    def code_type
      self.class
    end

    def error_type
      self.class::EXCEPTION_TYPE
    end

    def error!
      raise error_type.new("#{@code} #{@message}", self)
    end

    def value
      error! unless self.class <= HTTPSuccess
    end

    def reading_body sock, allow_body
      @socket = sock
      @body_exist = allow_body && self.class.body_permitted?
      begin
        yield
        read_body
      ensure
        @socket = nil
      end
    end

    def read_body dest = nil, &block
      if @read
        raise IOError, "#{self.class}#read_body called twice" if dest || block
        return @body
      end
      raise ArgumentError, 'both arg and block given for HTTP method' if dest && block
      raise IOError, 'attempt to read body out of block' if @socket.nil?
      if @body_exist
        text = @socket.read_all
        if block
          adapter = ReadAdapter.new block
          adapter << text
          @body = adapter
        elsif dest
          dest << text
          @body = dest
        else
          @body = text
        end
      else
        @body = nil
      end
      @read = true
      @body
    end

    def body
      read_body
    end

    def entity
      read_body
    end

    def self.read_new sock
      version, code, message = read_status_line sock
      response = response_class(code).new version, code, message
      each_response_header sock do |name, value|
        response.add_field name, value
      end
      response
    end

    def self.read_status_line sock
      line = sock.readline
      matched = %r{\AHTTP(?:/(\d+\.\d+))?\s+(\d\d\d)(?:\s+(.*))?\z}.match line
      raise HTTPBadResponse, "wrong status line: #{line.inspect}" if matched.nil?
      [matched[1], matched[2], matched[3]]
    end
    private_class_method :read_status_line

    def self.each_response_header sock
      name = nil
      value = nil
      loop do
        line = sock.readuntil("\n", true).sub(/\s+\z/, '')
        break if line.empty?
        if (line.start_with?(' ') || line.start_with?("\t")) && name
          value = "#{value} #{line.strip}"
        else
          yield name, value if name
          name, value = line.strip.split(/\s*:\s*/, 2)
          raise HTTPBadResponse, 'wrong header line format' if value.nil?
        end
      end
      yield name, value if name
    end
    private_class_method :each_response_header

    def self.response_class code
      CODE_TO_OBJ[code] || CODE_TO_OBJ[code[0, 1]] || HTTPUnknownResponse
    end
  end

  class HTTPUnknownResponse < HTTPResponse
    HAS_BODY = true
    EXCEPTION_TYPE = HTTPError
  end

  class HTTPInformation < HTTPResponse
    HAS_BODY = false
    EXCEPTION_TYPE = HTTPError
  end

  class HTTPSuccess < HTTPResponse
    HAS_BODY = true
    EXCEPTION_TYPE = HTTPError
  end

  class HTTPRedirection < HTTPResponse
    HAS_BODY = true
    EXCEPTION_TYPE = HTTPRetriableError
  end

  class HTTPClientError < HTTPResponse
    HAS_BODY = true
    EXCEPTION_TYPE = HTTPClientException
  end

  class HTTPServerError < HTTPResponse
    HAS_BODY = true
    EXCEPTION_TYPE = HTTPFatalError
  end

  class HTTPContinue < HTTPInformation
    HAS_BODY = false
  end

  class HTTPSwitchProtocol < HTTPInformation
    HAS_BODY = false
  end

  class HTTPProcessing < HTTPInformation
    HAS_BODY = false
  end

  class HTTPEarlyHints < HTTPInformation
    HAS_BODY = false
  end

  class HTTPOK < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPCreated < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPAccepted < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPNonAuthoritativeInformation < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPNoContent < HTTPSuccess
    HAS_BODY = false
  end

  class HTTPResetContent < HTTPSuccess
    HAS_BODY = false
  end

  class HTTPPartialContent < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPMultiStatus < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPAlreadyReported < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPIMUsed < HTTPSuccess
    HAS_BODY = true
  end

  class HTTPMultipleChoices < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPMovedPermanently < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPFound < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPSeeOther < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPNotModified < HTTPRedirection
    HAS_BODY = false
  end

  class HTTPUseProxy < HTTPRedirection
    HAS_BODY = false
  end

  class HTTPTemporaryRedirect < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPPermanentRedirect < HTTPRedirection
    HAS_BODY = true
  end

  class HTTPBadRequest < HTTPClientError
    HAS_BODY = true
  end

  class HTTPUnauthorized < HTTPClientError
    HAS_BODY = true
  end

  class HTTPPaymentRequired < HTTPClientError
    HAS_BODY = true
  end

  class HTTPForbidden < HTTPClientError
    HAS_BODY = true
  end

  class HTTPNotFound < HTTPClientError
    HAS_BODY = true
  end

  class HTTPMethodNotAllowed < HTTPClientError
    HAS_BODY = true
  end

  class HTTPNotAcceptable < HTTPClientError
    HAS_BODY = true
  end

  class HTTPProxyAuthenticationRequired < HTTPClientError
    HAS_BODY = true
  end

  class HTTPRequestTimeout < HTTPClientError
    HAS_BODY = true
  end

  class HTTPConflict < HTTPClientError
    HAS_BODY = true
  end

  class HTTPGone < HTTPClientError
    HAS_BODY = true
  end

  class HTTPLengthRequired < HTTPClientError
    HAS_BODY = true
  end

  class HTTPPreconditionFailed < HTTPClientError
    HAS_BODY = true
  end

  class HTTPPayloadTooLarge < HTTPClientError
    HAS_BODY = true
  end

  class HTTPURITooLong < HTTPClientError
    HAS_BODY = true
  end

  class HTTPUnsupportedMediaType < HTTPClientError
    HAS_BODY = true
  end

  class HTTPRangeNotSatisfiable < HTTPClientError
    HAS_BODY = true
  end

  class HTTPExpectationFailed < HTTPClientError
    HAS_BODY = true
  end

  class HTTPMisdirectedRequest < HTTPClientError
    HAS_BODY = true
  end

  class HTTPUnprocessableEntity < HTTPClientError
    HAS_BODY = true
  end

  class HTTPLocked < HTTPClientError
    HAS_BODY = true
  end

  class HTTPFailedDependency < HTTPClientError
    HAS_BODY = true
  end

  class HTTPTooEarly < HTTPClientError
    HAS_BODY = true
  end

  class HTTPUpgradeRequired < HTTPClientError
    HAS_BODY = true
  end

  class HTTPPreconditionRequired < HTTPClientError
    HAS_BODY = true
  end

  class HTTPTooManyRequests < HTTPClientError
    HAS_BODY = true
  end

  class HTTPRequestHeaderFieldsTooLarge < HTTPClientError
    HAS_BODY = true
  end

  class HTTPUnavailableForLegalReasons < HTTPClientError
    HAS_BODY = true
  end

  class HTTPInternalServerError < HTTPServerError
    HAS_BODY = true
  end

  class HTTPNotImplemented < HTTPServerError
    HAS_BODY = true
  end

  class HTTPBadGateway < HTTPServerError
    HAS_BODY = true
  end

  class HTTPServiceUnavailable < HTTPServerError
    HAS_BODY = true
  end

  class HTTPGatewayTimeout < HTTPServerError
    HAS_BODY = true
  end

  class HTTPVersionNotSupported < HTTPServerError
    HAS_BODY = true
  end

  class HTTPVariantAlsoNegotiates < HTTPServerError
    HAS_BODY = true
  end

  class HTTPInsufficientStorage < HTTPServerError
    HAS_BODY = true
  end

  class HTTPLoopDetected < HTTPServerError
    HAS_BODY = true
  end

  class HTTPNotExtended < HTTPServerError
    HAS_BODY = true
  end

  class HTTPNetworkAuthenticationRequired < HTTPServerError
    HAS_BODY = true
  end

  class HTTPResponse
    CODE_TO_OBJ = {
      '1' => HTTPInformation,
      '2' => HTTPSuccess,
      '3' => HTTPRedirection,
      '4' => HTTPClientError,
      '5' => HTTPServerError,
      '100' => HTTPContinue,
      '101' => HTTPSwitchProtocol,
      '102' => HTTPProcessing,
      '103' => HTTPEarlyHints,
      '200' => HTTPOK,
      '201' => HTTPCreated,
      '202' => HTTPAccepted,
      '203' => HTTPNonAuthoritativeInformation,
      '204' => HTTPNoContent,
      '205' => HTTPResetContent,
      '206' => HTTPPartialContent,
      '207' => HTTPMultiStatus,
      '208' => HTTPAlreadyReported,
      '226' => HTTPIMUsed,
      '300' => HTTPMultipleChoices,
      '301' => HTTPMovedPermanently,
      '302' => HTTPFound,
      '303' => HTTPSeeOther,
      '304' => HTTPNotModified,
      '305' => HTTPUseProxy,
      '307' => HTTPTemporaryRedirect,
      '308' => HTTPPermanentRedirect,
      '400' => HTTPBadRequest,
      '401' => HTTPUnauthorized,
      '402' => HTTPPaymentRequired,
      '403' => HTTPForbidden,
      '404' => HTTPNotFound,
      '405' => HTTPMethodNotAllowed,
      '406' => HTTPNotAcceptable,
      '407' => HTTPProxyAuthenticationRequired,
      '408' => HTTPRequestTimeout,
      '409' => HTTPConflict,
      '410' => HTTPGone,
      '411' => HTTPLengthRequired,
      '412' => HTTPPreconditionFailed,
      '413' => HTTPPayloadTooLarge,
      '414' => HTTPURITooLong,
      '415' => HTTPUnsupportedMediaType,
      '416' => HTTPRangeNotSatisfiable,
      '417' => HTTPExpectationFailed,
      '421' => HTTPMisdirectedRequest,
      '422' => HTTPUnprocessableEntity,
      '423' => HTTPLocked,
      '424' => HTTPFailedDependency,
      '425' => HTTPTooEarly,
      '426' => HTTPUpgradeRequired,
      '428' => HTTPPreconditionRequired,
      '429' => HTTPTooManyRequests,
      '431' => HTTPRequestHeaderFieldsTooLarge,
      '451' => HTTPUnavailableForLegalReasons,
      '500' => HTTPInternalServerError,
      '501' => HTTPNotImplemented,
      '502' => HTTPBadGateway,
      '503' => HTTPServiceUnavailable,
      '504' => HTTPGatewayTimeout,
      '505' => HTTPVersionNotSupported,
      '506' => HTTPVariantAlsoNegotiates,
      '507' => HTTPInsufficientStorage,
      '508' => HTTPLoopDetected,
      '510' => HTTPNotExtended,
      '511' => HTTPNetworkAuthenticationRequired
    }
  end

  HTTPRequestURITooLong = HTTPURITooLong
  HTTPRequestEntityTooLarge = HTTPPayloadTooLarge
  HTTPRequestedRangeNotSatisfiable = HTTPRangeNotSatisfiable
  HTTPRequestTimeOut = HTTPRequestTimeout
  HTTPGatewayTimeOut = HTTPGatewayTimeout
  HTTPMovedTemporarily = HTTPFound
  HTTPMultipleChoice = HTTPMultipleChoices
  HTTPInformationCode = HTTPInformation
  HTTPSuccessCode = HTTPSuccess
  HTTPRedirectionCode = HTTPRedirection
  HTTPRetriableCode = HTTPRedirection
  HTTPClientErrorCode = HTTPClientError
  HTTPFatalErrorCode = HTTPClientError
  HTTPServerErrorCode = HTTPServerError

  class HTTP < Protocol
    HTTPVersion = '1.1'

    class Get < HTTPRequest
      METHOD = 'GET'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Head < HTTPRequest
      METHOD = 'HEAD'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = false
    end

    class Post < HTTPRequest
      METHOD = 'POST'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Put < HTTPRequest
      METHOD = 'PUT'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Delete < HTTPRequest
      METHOD = 'DELETE'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Options < HTTPRequest
      METHOD = 'OPTIONS'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Trace < HTTPRequest
      METHOD = 'TRACE'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Patch < HTTPRequest
      METHOD = 'PATCH'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Propfind < HTTPRequest
      METHOD = 'PROPFIND'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Proppatch < HTTPRequest
      METHOD = 'PROPPATCH'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Mkcol < HTTPRequest
      METHOD = 'MKCOL'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Copy < HTTPRequest
      METHOD = 'COPY'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Move < HTTPRequest
      METHOD = 'MOVE'
      REQUEST_HAS_BODY = false
      RESPONSE_HAS_BODY = true
    end

    class Lock < HTTPRequest
      METHOD = 'LOCK'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    class Unlock < HTTPRequest
      METHOD = 'UNLOCK'
      REQUEST_HAS_BODY = true
      RESPONSE_HAS_BODY = true
    end

    def self.default_port
      80
    end

    def self.http_default_port
      80
    end

    def self.https_default_port
      443
    end

    def self.socket_type
      BufferedIO
    end

    def self.version_1_2
      true
    end

    def self.version_1_2?
      true
    end

    def self.version_1_1?
      false
    end

    def self.is_version_1_2?
      version_1_2?
    end

    def self.is_version_1_1?
      version_1_1?
    end

    # The subclass every connection through one proxy is made from, which
    # carries the proxy it reaches in class-level state.
    def self.Proxy p_addr = :no_proxy, p_port = nil, p_user = nil, p_pass = nil
      return self if p_addr.nil? || p_addr == :no_proxy
      proxy = Class.new self
      proxy.instance_variable_set :@is_proxy_class, true
      proxy.instance_variable_set :@proxy_address, p_addr
      proxy.instance_variable_set :@proxy_port, p_port || default_port
      proxy.instance_variable_set :@proxy_user, p_user
      proxy.instance_variable_set :@proxy_pass, p_pass
      proxy
    end

    def self.proxy_class?
      @is_proxy_class ? true : false
    end

    def self.proxy_address
      @proxy_address
    end

    def self.proxy_port
      @proxy_port
    end

    def self.proxy_user
      @proxy_user
    end

    def self.proxy_pass
      @proxy_pass
    end

    def self.newobj address, port = nil
      instance = allocate
      instance.__send__ :initialize, address, port
      instance
    end

    def self.new address, port = nil, p_addr = :no_proxy, p_port = nil, p_user = nil, p_pass = nil
      if p_addr == :no_proxy && proxy_class?
        p_addr = proxy_address
        p_port = proxy_port
        p_user = proxy_user
        p_pass = proxy_pass
      end
      instance = allocate
      instance.__send__ :initialize, address, port, p_addr, p_port, p_user, p_pass
      instance
    end

    def self.start address, *args, &block
      port = args.shift if args.first.is_a?(Integer) || args.first.nil?
      options = args.last.is_a?(Hash) ? args.pop : {}
      instance = new address, port
      instance.use_ssl = true if options[:use_ssl]
      instance.open_timeout = options[:open_timeout] if options[:open_timeout]
      instance.read_timeout = options[:read_timeout] if options[:read_timeout]
      instance.start(&block)
    end

    def self.get_response uri, headers = nil, &block
      target = uri.is_a?(String) ? URI.parse(uri) : uri
      start target.host, target.port do |http|
        http.request Get.new(target.request_uri, headers), &block
      end
    end

    def self.get uri, headers = nil
      get_response(uri, headers).body
    end

    def self.get_print uri, headers = nil
      get_response uri, headers do |response|
        print response.body
      end
      nil
    end

    def self.post uri, data, headers = nil
      start uri.host, uri.port do |http|
        http.post uri.request_uri, data, headers
      end
    end

    def self.post_form uri, params
      request = Post.new uri.request_uri
      request.form_data = params
      start uri.host, uri.port do |http|
        http.request request
      end
    end

    def initialize address, port = nil, p_addr = :no_proxy, p_port = nil, p_user = nil, p_pass = nil
      @address = address
      @port = port || HTTP.default_port
      if p_addr.nil? || p_addr == :no_proxy
        @proxy_address = nil
        @proxy_port = nil
        @proxy_user = nil
        @proxy_pass = nil
      else
        @proxy_address = p_addr
        @proxy_port = p_port || HTTP.default_port
        @proxy_user = p_user
        @proxy_pass = p_pass
      end
      @started = false
      @socket = nil
      @use_ssl = false
      @open_timeout = 60
      @read_timeout = 60
      @write_timeout = 60
      @continue_timeout = nil
      @keep_alive_timeout = 2
      @close_on_empty_response = false
      @debug_output = nil
      @local_host = nil
      @local_port = nil
    end

    private :initialize

    attr_reader :address
    attr_reader :port
    attr_reader :proxy_address
    attr_reader :proxy_port
    attr_reader :proxy_user
    attr_reader :proxy_pass
    attr_accessor :open_timeout
    attr_accessor :read_timeout
    attr_accessor :write_timeout
    attr_accessor :continue_timeout
    attr_accessor :keep_alive_timeout
    attr_accessor :close_on_empty_response
    attr_accessor :local_host
    attr_accessor :local_port

    def use_ssl?
      @use_ssl
    end

    def use_ssl= flag
      raise IOError, 'use_ssl value changed, but session already started' if @started
      @use_ssl = flag
    end

    def set_debug_output output
      @debug_output = output
    end

    def inspect
      "#<#{self.class} #{@address}:#{@port} open=#{@started}>"
    end

    def started?
      @started
    end

    def active?
      @started
    end

    def proxy?
      !@proxy_address.nil?
    end

    def proxy_class?
      self.class.proxy_class?
    end

    def proxyaddr
      @proxy_address
    end

    def proxyport
      @proxy_port
    end

    def start
      raise IOError, 'HTTP session already opened' if @started
      do_start
      return self unless block_given?
      begin
        yield self
      ensure
        finish
      end
    end

    def do_start
      require 'socket'
      @socket = BufferedIO.new TCPSocket.new(conn_address, conn_port),
                               read_timeout: @read_timeout,
                               write_timeout: @write_timeout,
                               continue_timeout: @continue_timeout,
                               debug_output: @debug_output
      @started = true
    end
    private :do_start

    def conn_address
      proxy? ? @proxy_address : @address
    end
    private :conn_address

    def conn_port
      proxy? ? @proxy_port : @port
    end
    private :conn_port

    def finish
      raise IOError, 'HTTP session not yet started' unless @started
      @socket.close if @socket && !@socket.closed?
      @socket = nil
      @started = false
      nil
    end

    def request request, body = nil, &block
      start unless @started
      request.set_body_internal body if body
      request.exec @socket, HTTPVersion, edit_path(request.path)
      response = HTTPResponse.read_new @socket
      response.uri = request.uri
      response.reading_body @socket, request.response_body_permitted? do
        yield response if block
      end
      response
    end

    def edit_path path
      proxy? && !use_ssl? ? "http://#{addr_port}#{path}" : path
    end
    private :edit_path

    def addr_port
      default = use_ssl? ? HTTP.https_default_port : HTTP.http_default_port
      @port == default ? @address : "#{@address}:#{@port}"
    end
    private :addr_port

    def send_entity path, data, initheader, dest, type, &block
      request = type.new path, initheader
      request.set_body_internal data
      request(request, nil, &block)
    end
    private :send_entity

    def get path, initheader = nil, &block
      request Get.new(path, initheader), nil, &block
    end

    def head path, initheader = nil
      request Head.new(path, initheader)
    end

    def post path, data, initheader = nil, &block
      send_entity path, data, initheader, nil, Post, &block
    end

    def put path, data, initheader = nil
      send_entity path, data, initheader, nil, Put
    end

    def patch path, data, initheader = nil, &block
      send_entity path, data, initheader, nil, Patch, &block
    end

    def delete path, initheader = nil
      request Delete.new(path, initheader)
    end

    def options path, initheader = nil
      request Options.new(path, initheader)
    end

    def trace path, initheader = nil
      request Trace.new(path, initheader)
    end
  end
end
