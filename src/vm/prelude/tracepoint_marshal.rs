pub(super) const SOURCE: &str = r##"
class TracePoint
  KNOWN_EVENTS = [:line, :call, :return, :c_call, :c_return, :class, :end,
                  :b_call, :b_return, :raise, :rescue, :thread_begin,
                  :thread_end, :fiber_switch, :script_compiled, :a_call,
                  :a_return]

  def self.new(*events, &block)
    # An event may be named with a String or with anything answering
    # `to_sym`, and only a Symbol comes back from that.
    events = events.map do |event|
      named = if event.is_a?(Symbol)
        event
      elsif event.respond_to?(:to_sym)
        event.to_sym
      else
        raise TypeError, "#{event.inspect} is not a symbol nor a string"
      end
      unless named.is_a?(Symbol)
        raise TypeError, "#{event.inspect} is not a symbol nor a string"
      end
      named
    end
    events.each do |event|
      unless KNOWN_EVENTS.include?(event)
        raise ArgumentError, "unknown event: #{event}"
      end
    end
    raise ArgumentError, "must be called with a block" if block.nil?
    made = allocate
    made.send(:__set_up__, events, block)
    made
  end

  def self.trace(*events, &block)
    made = new(*events, &block)
    made.enable
    made
  end

  def __set_up__(events, block)
    @events = events.empty? ? KNOWN_EVENTS : events
    @block = block
    @enabled = false
    @handling = nil
    self
  end
  private :__set_up__

  def enabled?
    @enabled == true
  end

  # With a block, the trace is on for the length of it and the block's value
  # is the answer. Without one, the answer is what the switch was before.
  def enable(target: nil, target_line: nil, target_thread: nil, &block)
    was = enabled?
    @enabled = true
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      __register__
    end
  end

  def disable(&block)
    was = enabled?
    @enabled = false
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      __register__
    end
  end

  # Ruby switches a trace off while its own handler runs, so an event the
  # handler causes does not call it again. `allow_reentry` lifts that for the
  # length of a block.
  # Ruby switches a trace off while its own handler runs, so an event the
  # handler causes does not call it again. `allow_reentry` lifts that for the
  # length of a block, and is refused outside a handler.
  def self.allow_reentry
    raise RuntimeError, "allow_reentry is not allowed outside of a trace" unless __tracing__
    __reentrant__ { yield }
  end

  def event
    __reading__(:event)
  end

  def lineno
    __reading__(:lineno)
  end

  def path
    __reading__(:path)
  end

  def self
    __reading__(:self)
  end

  def method_id
    __reading__(:method_id)
  end

  def return_value
    __reading__(:return_value)
  end

  def callee_id
    __reading__(:callee_id)
  end

  def defined_class
    __reading__(:defined_class)
  end

  # The scope the event fired in, as the Binding that reads its locals.
  def binding
    __reading__(:binding)
  end

  # The source a `script_compiled` event compiled, where it came from a
  # string rather than a file.
  def eval_script
    __reading__(:eval_script)
  end

  # The exception a `raise` or a `rescue` event is about.
  def raised_exception
    __reading__(:raised_exception)
  end

  # What the block a `b_call` or `b_return` event is about takes.
  def parameters
    __reading__(:parameters)
  end

  def inspect
    return "#<TracePoint:disabled>" if @handling.nil?
    "#<TracePoint:#{@handling["event"]}@#{@handling["path"]}:#{@handling["lineno"]}>"
  end

  # What the event being handled says about itself. Nothing is being handled
  # outside a handler, which is what Ruby reports.
  def __reading__(name)
    raise RuntimeError, "access from outside" if @handling.nil?
    @handling[name.to_s]
  end
  private :__reading__

  def __handle__(details)
    held = @handling
    @handling = details
    begin
      @block.call(self)
    ensure
      @handling = held
    end
  end

  def __wants__(event)
    @events.include?(event)
  end
end

# The format version `Marshal.dump` writes and `Marshal.load` reads.
module Marshal
  MAJOR_VERSION = 4
  MINOR_VERSION = 8

  # The bytes an object is written as, opening with the format version.
  def self.dump(object, target = nil, _limit = nil)
    target = nil if target.is_a? Integer
    written = ([MAJOR_VERSION, MINOR_VERSION] + Writer.new.bytes_for(object)).pack "C*"
    written = written.force_encoding Encoding::BINARY
    return written if target.nil?
    target.write written
    target
  end

  # The object a run of marshalled bytes spells.
  def self.load(source, handler = nil, freeze: false)
    source = source.read unless source.is_a? String
    made = Reader.new(source, handler).read_document
    freeze ? made.freeze : made
  end

  def self.restore(source, handler = nil, freeze: false)
    load source, handler, freeze: freeze
  end

  # The bytes a whole number is written as: one byte for a small one, and a
  # count followed by the bytes themselves for anything wider.
  def self.__long_bytes__(value)
    return [0] if value == 0
    return [value + 5] if value > 0 && value < 123
    return [(value - 5) & 0xff] if value < 0 && value > -124
    held = value
    bytes = []
    counted = 0
    1.upto(8) do |index|
      bytes << (held & 0xff)
      held = held >> 8
      counted = index
      break if held == 0 || held == -1
    end
    [held == -1 ? (-counted) & 0xff : counted] + bytes
  end

  # How Marshal spells a Float: the shortest run of digits that reads back as
  # the same number, in exponent form when the point sits far from them.
  def self.__float_text__(value)
    return "nan" if value.nan?
    return value < 0 ? "-inf" : "inf" if value.infinite?
    sign = value.to_s.start_with?("-") ? "-" : ""
    return "#{sign}0" if value == 0.0
    digits, point = __digits_of__(value.abs)
    return "#{sign}#{__exponent_form__(digits, point)}" if point < -3 || point > 16
    return "#{sign}0.#{"0" * -point}#{digits}" if point <= 0
    return "#{sign}#{digits}#{"0" * (point - digits.length)}" if point >= digits.length
    "#{sign}#{digits[0, point]}.#{digits[point..]}"
  end

  # The digits of a Float and where the point sits among them, read off the
  # shortest spelling that reads back as the same number.
  def self.__digits_of__(value)
    spelled = value.to_s
    if spelled.include? "e"
      mantissa, exponent = spelled.split "e"
      digits = mantissa.delete "."
      return [__trimmed_digits__(digits), exponent.to_i + 1]
    end
    whole, fraction = spelled.split "."
    fraction = "" if fraction.nil?
    if whole == "0"
      leading = fraction.length - fraction.sub(/\A0+/, "").length
      return ["0", 1] if fraction.sub(/\A0+/, "").empty?
      [__trimmed_digits__(fraction[leading..]), -leading]
    else
      [__trimmed_digits__(whole + fraction), whole.length]
    end
  end

  # A run of digits with the trailing zeros dropped, which are not part of the
  # shortest spelling.
  def self.__trimmed_digits__(digits)
    trimmed = digits.sub(/0+\z/, "")
    trimmed.empty? ? "0" : trimmed
  end

  # One digit, the rest after a point, and the power of ten they stand at.
  def self.__exponent_form__(digits, point)
    body = digits.length > 1 ? "#{digits[0]}.#{digits[1..]}" : digits
    "#{body}e#{point - 1}"
  end

  # Walks an object and writes the bytes Marshal spells it in.
  class Writer
    def initialize
      @symbols = {}
      @objects = {}
      @out = []
    end

    def bytes_for(object)
      write object
      @out
    end

    private

    def text(spelled)
      spelled.bytes.each { |held| @out << held }
    end

    def long(value)
      Marshal.__long_bytes__(value).each { |held| @out << held }
    end

    def symbol(name)
      spelled = name.to_s
      if @symbols.key? spelled
        text ";"
        long @symbols[spelled]
        return
      end
      @symbols[spelled] = @symbols.size
      text ":"
      long spelled.bytesize
      text spelled
    end

    def remember(object)
      @objects[object.object_id] = @objects.size
    end

    def linked(object)
      return false unless @objects.key? object.object_id
      text "@"
      long @objects[object.object_id]
      true
    end

    def write(object)
      case object
      when nil then text "0"
      when true then text "T"
      when false then text "F"
      when Symbol then symbol object
      when Integer then write_integer object
      else
        return if linked object
        write_held object
      end
    end

    def write_integer(value)
      if value >= -1073741824 && value <= 1073741823
        text "i"
        long value
        return
      end
      remember value
      text "l"
      text(value < 0 ? "-" : "+")
      held = value.abs
      words = []
      while held > 0
        words << (held & 0xffff)
        held = held >> 16
      end
      words = [0] if words.empty?
      long words.length
      words.each do |word|
        @out << (word & 0xff)
        @out << ((word >> 8) & 0xff)
      end
    end

    def write_held(object)
      case object
      when Float
        remember object
        text "f"
        spelled = Marshal.__float_text__(object)
        long spelled.bytesize
        text spelled
      when String then write_string object
      when Array
        remember object
        text "["
        long object.length
        object.each { |item| write item }
      when Hash
        remember object
        text "{"
        long object.length
        object.each { |key, value| write key; write value }
      when Class
        remember object
        text "c"
        long object.name.bytesize
        text object.name
      when Module
        remember object
        text "m"
        long object.name.bytesize
        text object.name
      else write_instance object
      end
    end

    def write_string(object)
      remember object
      named = object.encoding.name
      if named == "ASCII-8BIT"
        text '"'
        long object.bytesize
        text object
        return
      end
      text "I"
      text '"'
      long object.bytesize
      text object
      long 1
      if named == "UTF-8"
        symbol :E
        write true
      elsif named == "US-ASCII"
        symbol :E
        write false
      else
        symbol :encoding
        text '"'
        long named.bytesize
        text named
      end
    end

    def write_instance(object)
      if object.respond_to? :marshal_dump, true
        remember object
        text "U"
        symbol object.class.name
        write object.marshal_dump
        return
      end
      if object.respond_to? :_dump, true
        held = object.send :_dump, -1
        remember object
        names = held.instance_variables
        text "I" unless names.empty?
        text "u"
        symbol object.class.name
        long held.bytesize
        text held
        unless names.empty?
          long names.length
          names.each do |name|
            symbol name
            write held.instance_variable_get(name)
          end
        end
        return
      end
      remember object
      text "o"
      symbol object.class.name
      names = object.instance_variables
      long names.length
      names.each do |name|
        symbol name
        write object.instance_variable_get(name)
      end
    end
  end

  # Reads the bytes Marshal spells an object in, putting it back together.
  class Reader
    def initialize(source, handler = nil)
      @bytes = source.bytes
      @at = 0
      @symbols = []
      @objects = []
      @handler = handler
    end

    def read_document
      major = next_byte
      minor = next_byte
      if major != Marshal::MAJOR_VERSION || minor > Marshal::MINOR_VERSION
        raise TypeError, "incompatible marshal file format (can't be read)"
      end
      made = read
      @handler.call made unless @handler.nil?
      made
    end

    private

    def next_byte
      held = @bytes[@at]
      raise ArgumentError, "marshal data too short" if held.nil?
      @at += 1
      held
    end

    def signed_byte
      held = next_byte
      held > 127 ? held - 256 : held
    end

    def read_long
      opening = signed_byte
      return 0 if opening == 0
      if opening > 0
        return opening - 5 if opening > 4
        value = 0
        opening.times { |index| value |= next_byte << (8 * index) }
        return value
      end
      return opening + 5 if opening < -4
      value = -1
      (-opening).times do |index|
        value &= ~(0xff << (8 * index))
        value |= next_byte << (8 * index)
      end
      value
    end

    def read_bytes(count)
      held = @bytes[@at, count]
      @at += count
      held.pack("C*").force_encoding Encoding::BINARY
    end

    def remember(object)
      @objects << object
      object
    end

    def read
      case next_byte.chr
      when "0" then nil
      when "T" then true
      when "F" then false
      when "i" then read_long
      when "f" then remember read_float
      when ":" then read_symbol
      when ";" then @symbols[read_long]
      when "@" then @objects[read_long]
      when '"' then remember read_bytes(read_long)
      when "I" then read_with_variables
      when "[" then read_array
      when "{" then read_hash
      when "l" then remember read_bignum
      when "o" then read_instance
      when "U" then read_user_marshal
      when "u" then read_user_defined
      when "c" then remember named_class(read_bytes(read_long))
      when "m" then remember named_class(read_bytes(read_long))
      else raise TypeError, "dump format error"
      end
    end

    def named_class(name)
      Object.const_get name.force_encoding(Encoding::UTF_8)
    end

    def read_symbol
      name = read_bytes(read_long).force_encoding(Encoding::UTF_8).to_sym
      @symbols << name
      name
    end

    def read_float
      spelled = read_bytes read_long
      case spelled
      when "nan" then 0.0 / 0.0
      when "inf" then 1.0 / 0.0
      when "-inf" then -1.0 / 0.0
      else spelled.to_f
      end
    end

    def read_bignum
      sign = next_byte.chr
      words = read_long
      value = 0
      words.times do |index|
        low = next_byte
        high = next_byte
        value |= (low | (high << 8)) << (16 * index)
      end
      sign == "-" ? -value : value
    end

    def read_array
      made = []
      @objects << made
      read_long.times { made << read }
      made
    end

    def read_hash
      made = {}
      @objects << made
      read_long.times do
        key = read
        made[key] = read
      end
      made
    end

    def read_with_variables
      # A record written by an object's own `_dump` carries its variables on
      # the run of bytes rather than on the object, since the object is not
      # built until those bytes are read back.
      if @bytes[@at] == "u".ord
        @at += 1
        return read_user_defined true
      end
      made = read
      read_long.times do
        name = read
        value = read
        apply_variable made, name, value
      end
      made
    end

    def apply_variable(made, name, value)
      if name == :E && made.is_a?(String)
        made.force_encoding(value ? Encoding::UTF_8 : Encoding::US_ASCII)
      elsif name == :encoding && made.is_a?(String)
        made.force_encoding value
      else
        made.instance_variable_set name, value
      end
    end

    def read_instance
      made = named_class(read.to_s).allocate
      @objects << made
      read_long.times do
        name = read
        made.instance_variable_set name, read
      end
      made
    end

    def read_user_marshal
      made = named_class(read.to_s).allocate
      @objects << made
      made.send :marshal_load, read
      made
    end

    def read_user_defined(carries_variables = false)
      klass = named_class read.to_s
      data = read_bytes read_long
      if carries_variables
        read_long.times do
          name = read
          value = read
          apply_variable data, name, value
        end
      end
      remember klass.send(:_load, data)
    end
  end
end
"##;
