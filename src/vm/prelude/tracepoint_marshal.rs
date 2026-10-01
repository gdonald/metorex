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
    @target_place = nil
    @target_line = nil
    @target_thread = nil
    self
  end
  private :__set_up__

  def enabled?
    @enabled == true
  end

  # The events code of each kind can report when a trace is aimed at it.
  BLOCK_EVENTS = [:line, :b_call, :b_return, :class, :end, :raise, :rescue, :c_call, :c_return]
  METHOD_EVENTS = BLOCK_EVENTS + [:call, :return]
  # The events a body with no statements reports nothing for.
  STATEMENT_EVENTS = [:line, :class, :end, :raise, :rescue, :c_call, :c_return]

  # With a block, the trace is on for the length of it and the block's value
  # is the answer. Without one, the answer is what the switch was before. A
  # trace switched on around a block only hears the thread that switched it
  # on, unless it names a thread or aims at a method or a block.
  def enable(target: nil, target_line: nil, target_thread: :__unset__, &block)
    raise ArgumentError, "can't nest-enable a targeting TracePoint" unless @target_place.nil?
    if target.nil?
      raise ArgumentError, "only target_line is specified" unless target_line.nil?
    else
      place = __target_place__(target)
      raise ArgumentError, "can't nest-enable a targeting TracePoint" if enabled?
      line = __target_line__(target, target_line)
    end
    if target_thread == :__unset__
      target_thread = !block.nil? && target.nil? ? Thread.current : nil
    end
    was = enabled?
    @enabled = true
    @target_place = place
    @target_line = line
    @target_thread = target_thread
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      @target_place = nil
      @target_line = nil
      __register__
    end
  end

  def disable(&block)
    unless block.nil? || @target_place.nil?
      raise ArgumentError, "can't disable a targeting TracePoint in a block"
    end
    was = enabled?
    @enabled = false
    @target_place = nil
    @target_line = nil
    __register__
    return was if block.nil?
    begin
      block.call
    ensure
      @enabled = was
      __register__
    end
  end

  # Where the code a trace is aimed at was written, as the file and the line
  # it opened on.
  def __target_place__(target)
    unless target.is_a?(Method) || target.is_a?(UnboundMethod) || target.is_a?(Proc)
      raise ArgumentError, "specified target is not supported"
    end
    place = target.source_location
    raise ArgumentError, "specified target is not supported" if place.nil?
    place
  end
  private :__target_place__

  # The one line a trace aimed at `target` reports `:line` events for, or
  # nil for every line. Refused where the target has nothing to report.
  def __target_line__(target, target_line)
    lines = __code_lines__(target)
    available = target.is_a?(Proc) ? BLOCK_EVENTS : METHOD_EVENTS
    available -= STATEMENT_EVENTS if lines.empty?
    unless target_line.nil?
      unless @events.include?(:line)
        raise ArgumentError, "target_line is specified, but line event is not specified"
      end
      target_line = if target_line.is_a?(Integer)
        target_line
      elsif target_line.respond_to?(:to_int)
        target_line.to_int
      else
        raise TypeError, "no implicit conversion of #{target_line.class} into Integer"
      end
      available = [] if lines.empty? || target_line < lines.first
    end
    raise ArgumentError, "can not enable any hooks" if (@events & available).empty?
    target_line
  end
  private :__target_line__

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
    return "#<TracePoint:#{enabled? ? "enabled" : "disabled"}>" if @handling.nil?
    event = @handling["event"]
    place = "#{@handling["path"]}:#{@handling["lineno"]}"
    case event
    when :call, :c_call, :return, :c_return
      "#<TracePoint:#{event} '#{@handling["method_id"]}' #{place}>"
    when :thread_begin, :thread_end
      "#<TracePoint:#{event} #{Thread.current.inspect}>"
    else
      "#<TracePoint:#{event} #{place}>"
    end
  end

  # What the event being handled says about itself. Nothing is being handled
  # outside a handler, which is what Ruby reports.
  def __reading__(name)
    raise RuntimeError, "access from outside" if @handling.nil?
    @handling[name.to_s]
  end
  private :__reading__

  def __handle__(details)
    return unless @target_thread.nil? || Thread.current.equal?(@target_thread)
    return unless @target_place.nil? || details["__within__"].include?(@target_place)
    return if !@target_line.nil? && (details["event"] != :line || details["lineno"] != @target_line)
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
  def self.dump(object, target = nil, limit = -1)
    if target.is_a? Integer
      limit = target
      target = nil
    end
    unless target.nil?
      raise TypeError, "instance of IO needed" unless target.respond_to? :write
      target.binmode if target.respond_to? :binmode
    end
    written = ([MAJOR_VERSION, MINOR_VERSION] + Writer.new(limit).bytes_for(object)).pack "C*"
    written = written.force_encoding Encoding::BINARY
    return written if target.nil?
    target.write written
    target
  end

  # The object a run of marshalled bytes spells.
  def self.load(source, handler = nil, freeze: false)
    bytes = if source.is_a? String
      source
    elsif source.respond_to? :to_str
      source.to_str
    elsif source.respond_to?(:read) && source.respond_to?(:getc)
      source.binmode if source.respond_to? :binmode
      held = source.read
      raise EOFError, "end of file reached" if held.nil? || held.empty?
      held
    else
      raise TypeError, "instance of IO needed"
    end
    Reader.new(bytes, handler, freeze).read_document
  end

  def self.restore(source, handler = nil, freeze: false)
    load source, handler, freeze: freeze
  end

  # The instance variables a program set on an object, leaving out the ones
  # the interpreter keeps its own state in.
  def self.__variables_of__(object)
    Kernel.instance_method(:instance_variables).bind_call(object).reject do |name|
      name.to_s.start_with? "@__"
    end
  end

  # Whether an object answers a method, asking the object itself when it
  # says, and taking a BasicObject that cannot say as answering nothing.
  def self.__responds__(object, name)
    object.respond_to? name, true
  rescue NoMethodError
    false
  end

  # The class or module a path names, looked up one part at a time.
  def self.__path_to_class__(path)
    path.to_s.split("::").inject(Object) do |held, part|
      raise ArgumentError, "#{path} does not refer to class/module" unless held.is_a? Module
      held.const_get part, false
    end
  rescue NameError
    raise ArgumentError, "undefined class/module #{path}"
  end

  # A run of bytes tagged with the encoding a variable written after it
  # names: `E` for UTF-8 or US-ASCII, or `encoding` for any other.
  def self.__encoded__(bytes, name, value)
    case name
    when :E then bytes.dup.force_encoding(value ? Encoding::UTF_8 : Encoding::US_ASCII)
    when :encoding then bytes.dup.force_encoding(value)
    else bytes
    end
  end

  # Set an instance variable on anything, a BasicObject included.
  def self.__set_variable__(made, name, value)
    Kernel.instance_method(:instance_variable_set).bind_call(made, name, value)
  end

  # A value frozen the way `freeze: true` asks: a class or a module is left
  # as it is, a String is swapped for the one frozen copy equal Strings
  # share, and anything else is frozen without calling a `freeze` of its own.
  def self.__frozen__(value)
    return value if value.is_a? Module
    return -value if value.instance_of? String
    Kernel.instance_method(:freeze).bind_call(value)
  end

  # A value read as a String, Array, Hash or Regexp, made an instance of the
  # subclass it was written as.
  def self.__as_subclass__(held_class, inner)
    if held_class.equal?(Hash) && inner.instance_of?(Hash)
      return inner.compare_by_identity
    end
    base = [String, Array, Hash, Regexp].find { |kind| inner.is_a? kind }
    raise ArgumentError, "dump format error (user class)" if base.nil? || !(held_class <= base)
    if base.equal? Regexp
      made = held_class.new(inner)
    else
      made = held_class.allocate
      made.__send__ :replace, inner
    end
    if base.equal? Hash
      made.default = inner.default unless inner.default.nil?
      made.compare_by_identity if inner.compare_by_identity?
    end
    __variables_of__(inner).each do |name|
      __set_variable__ made, name, inner.instance_variable_get(name)
    end
    made
  end

  # The classes whose instances are values of a kind of their own rather
  # than objects holding instance variables.
  def self.__value_classes__
    [String, Array, Hash, Regexp, Integer, Float, Symbol, IO, Range, Proc, Method]
  end

  # The instance variables a Time keeps its own state in, which its bytes and
  # the offset and zone written beside them already carry.
  TIME_STATE = [:@seconds, :@fraction, :@utc, :@offset, :@calendar, :@zone, :@zone_name, :@zone_object]

  # The classes whose objects hold what cannot be written as bytes.
  def self.__unwritable_classes__
    held = [Proc, Method, UnboundMethod, Binding, Thread, Mutex, ConditionVariable, Queue]
    held << Object.const_get(:StringIO) if Object.const_defined?(:StringIO)
    held
  end

  # Whether a whole number is written in place rather than as an object.
  def self.__fits_a_word__(value)
    value >= -1073741824 && value <= 1073741823
  end

  # The name a class is written under, which is the one it was given rather
  # than whatever an overridden `name` answers.
  def self.__class_path__(held)
    named = Module.instance_method(:name).bind_call(held)
    raise TypeError, "can't dump anonymous #{held.is_a?(Class) ? "class" : "module"} #{held}" if named.nil?
    named
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
    def initialize(limit = -1)
      @symbols = {}
      @objects = {}
      @out = []
      @limit = limit
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
      # A symbol spelled with more than ASCII carries its encoding with it.
      pairs = spelled.ascii_only? ? [] : encoding_pairs(spelled.encoding)
      text "I" unless pairs.empty?
      text ":"
      long spelled.bytesize
      text spelled
      write_pairs pairs
    end

    # The instance variables that stand for an encoding: `E` for UTF-8 and
    # US-ASCII, nothing for bytes, and the name for anything else.
    def encoding_pairs(encoding)
      case encoding.name
      when "ASCII-8BIT" then []
      when "UTF-8" then [[:E, true]]
      when "US-ASCII" then [[:E, false]]
      else [[:encoding, encoding.name.b]]
      end
    end

    def write_pairs(pairs, limit = @limit)
      return if pairs.empty?
      long pairs.length
      pairs.each do |name, value|
        symbol name
        write value, limit
      end
    end

    # The modules an object was extended with, most recently first. One with
    # no name cannot be found again when the object is read back.
    def write_extended(object)
      singleton = begin
        Kernel.instance_method(:singleton_class).bind_call(object)
      rescue TypeError
        return
      end
      extended = singleton.ancestors.drop(1).take_while { |held| !held.is_a?(Class) }
      extended.each do |held|
        named = Module.instance_method(:name).bind_call(held)
        raise TypeError, "can't dump anonymous class #{held}" if named.nil?
        text "e"
        symbol named
      end
    end

    # The class an object belongs to, when it is a subclass of the kind of
    # value it is written as.
    def write_user_class(object, base)
      return if object.class.equal? base
      text "C"
      symbol Marshal.__class_path__(object.class)
    end

    def remember(object)
      @objects[object.__id__] = @objects.size
    end

    def linked(object)
      return false unless @objects.key? object.__id__
      text "@"
      long @objects[object.__id__]
      true
    end

    # Every value is written one level deeper than the one holding it, and a
    # dump told how deep it may go refuses to go further.
    def write(object, limit = @limit)
      raise ArgumentError, "exceed depth limit" if limit == 0
      case object
      when nil then text "0"
      when true then text "T"
      when false then text "F"
      when Symbol then symbol object
      when Integer
        # A number too wide for a word is an object, which is written once
        # and linked after that.
        return if !Marshal.__fits_a_word__(object) && linked(object)
        write_integer object
      else
        return if linked object
        write_held object, limit - 1
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

    def write_held(object, limit)
      held_class = Kernel.instance_method(:class).bind_call(object)
      # An object that answers `marshal_dump` is written as whatever that
      # answers, under its own class.
      if Marshal.__responds__(object, :marshal_dump)
        remember object
        dumped = object.__send__ :marshal_dump
        text "U"
        symbol Marshal.__class_path__(held_class)
        write dumped, limit
        return
      end
      # One that answers `_dump` is written as the String that answers, with
      # that String's encoding and instance variables, and is remembered
      # after them.
      if Marshal.__responds__(object, :_dump)
        dumped = object.__send__ :_dump, limit
        raise TypeError, "_dump() must return string" unless dumped.is_a? String
        pairs = encoding_pairs(dumped.encoding)
        Marshal.__variables_of__(dumped).each do |name|
          pairs << [name, dumped.instance_variable_get(name)]
        end
        pairs.concat time_pairs(object) if object.is_a? Time
        text "I" unless pairs.empty?
        text "u"
        symbol Marshal.__class_path__(held_class)
        long dumped.bytesize
        text dumped
        write_pairs pairs, limit
        remember object
        return
      end
      case object
      when Float
        remember object
        text "f"
        spelled = Marshal.__float_text__(object)
        long spelled.bytesize
        text spelled
      when String, Regexp, Array, Hash then write_builtin object, limit
      when Class, Module then write_named object
      when Struct, Data then write_struct object, held_class, limit
      when Range then write_range object, held_class, limit
      when Exception then write_exception object, held_class, limit
      when *Marshal.__unwritable_classes__
        raise TypeError, "no _dump_data is defined for class #{held_class}"
      when IO, MatchData
        raise TypeError, "can't dump #{held_class}"
      else write_object object, held_class, limit
      end
    end

    # What a Time carries beside its own bytes: its instance variables, the
    # offset from UTC unless it is in UTC, and the name of its zone.
    def time_pairs(object)
      pairs = (Marshal.__variables_of__(object) - Marshal::TIME_STATE).map do |name|
        [name, object.instance_variable_get(name)]
      end
      # The bytes hold whole microseconds, and what is left below one travels
      # as nanoseconds, first as a fraction and then as three decimal digits.
      nano = object.subsec * 1_000_000_000 - object.usec * 1000
      unless nano.zero?
        pairs << [:nano_num, nano.numerator] << [:nano_den, nano.denominator]
        digits = nano.to_i
        unless digits.zero?
          ones = digits % 10
          tens = digits / 10 % 10
          hundreds = digits / 100
          submicro = (hundreds << 4 | tens).chr
          submicro << (ones << 4).chr unless ones.zero?
          pairs << [:submicro, submicro.force_encoding(Encoding::BINARY)]
        end
      end
      pairs << [:offset, object.utc_offset] unless object.utc?
      # A zone given as an object is written under the name it answers.
      zone = object.zone
      zone = zone.name unless zone.nil? || zone.is_a?(String)
      pairs << [:zone, zone.to_s.dup.force_encoding(Encoding::US_ASCII)] unless zone.nil?
      pairs
    end

    # A Range, written as an object holding whether it leaves out its end,
    # and its two ends.
    def write_range(object, held_class, limit)
      remember object
      write_extended object
      text "o"
      symbol Marshal.__class_path__(held_class)
      pairs = [[:excl, object.exclude_end?], [:begin, object.begin], [:end, object.end]]
      long pairs.length
      pairs.each do |name, value|
        symbol name
        write value, limit
      end
    end

    # An Exception, written as an object holding its message, its backtrace,
    # its cause when it has one, and its own instance variables.
    def write_exception(object, held_class, limit)
      remember object
      write_extended object
      text "o"
      symbol Marshal.__class_path__(held_class)
      pairs = [[:mesg, object.__given_message__], [:bt, object.backtrace]]
      pairs << [:cause, object.cause] unless object.cause.nil?
      Marshal.__variables_of__(object).each do |name|
        pairs << [name, object.instance_variable_get(name)]
      end
      long pairs.length
      pairs.each do |name, value|
        symbol name
        write value, limit
      end
    end

    # A class or a module, written as the name it goes by, which is all that
    # is needed to find it again.
    def write_named(object)
      raise TypeError, "singleton class can't be dumped" if object.singleton_class?
      named = Marshal.__class_path__(object)
      remember object
      pairs = named.ascii_only? ? [] : encoding_pairs(named.encoding)
      text "I" unless pairs.empty?
      text(object.is_a?(Class) ? "c" : "m")
      long named.bytesize
      text named
      write_pairs pairs
    end

    # A Struct or a Data, written as its members and their values.
    def write_struct(object, held_class, limit)
      remember object
      members = object.to_h
      pairs = []
      Marshal.__variables_of__(object).each do |name|
        next if members.key? name.to_s.delete_prefix("@").to_sym
        pairs << [name, object.instance_variable_get(name)]
      end
      text "I" unless pairs.empty?
      write_extended object
      text "S"
      symbol Marshal.__class_path__(held_class)
      long members.length
      members.each do |member, value|
        symbol member
        write value, limit
      end
      write_pairs pairs, limit
    end

    # Any other object, written as its class and its instance variables. One
    # with methods or variables of its own on its singleton class cannot be
    # put back together, since only its class is written.
    def write_object(object, held_class, limit)
      singleton_methods = Kernel.instance_method(:singleton_methods).bind_call(object, false)
      singleton = Kernel.instance_method(:singleton_class).bind_call(object)
      unless singleton_methods.empty? && Marshal.__variables_of__(singleton).empty?
        raise TypeError, "singleton can't be dumped"
      end
      remember object
      write_extended object
      text "o"
      symbol Marshal.__class_path__(held_class)
      names = Marshal.__variables_of__(object)
      long names.length
      names.each do |name|
        symbol name
        write Kernel.instance_method(:instance_variable_get).bind_call(object, name), limit
      end
    end

    # A String, Regexp, Array or Hash, wrapped in what else it carries: its
    # encoding and instance variables, the modules it was extended with, and
    # its class when that is a subclass.
    def write_builtin(object, limit = @limit)
      if object.is_a?(Hash) && !object.default_proc.nil?
        raise TypeError, "can't dump hash with default proc"
      end
      remember object
      base = [String, Regexp, Array, Hash].find { |kind| object.is_a? kind }
      pairs = [String, Regexp].include?(base) ? encoding_pairs(object.encoding) : []
      Marshal.__variables_of__(object).each do |name|
        pairs << [name, object.instance_variable_get(name)]
      end
      text "I" unless pairs.empty?
      write_extended object
      write_user_class object, base
      if base.equal?(Hash) && object.compare_by_identity?
        text "C"
        symbol :Hash
      end
      if base.equal? String
        text '"'
        long object.bytesize
        text object
      elsif base.equal? Regexp
        text "/"
        long object.source.bytesize
        text object.source
        @out << (object.options & 0xff)
      elsif base.equal? Array
        text "["
        long object.length
        object.each { |item| write item, limit }
      else
        text(object.default.nil? ? "{" : "}")
        long object.length
        object.each { |key, value| write key, limit; write value, limit }
        write object.default, limit unless object.default.nil?
      end
      write_pairs pairs, limit
    end
  end

  # Reads the bytes Marshal spells an object in, putting it back together.
  # Every value is finished through `leave`, which freezes it when asked and
  # hands it to the proc when there is one.
  class Reader
    def initialize(source, handler = nil, freeze = false)
      @bytes = source.bytes
      @at = 0
      @symbols = []
      @objects = []
      @building = {}
      @handler = handler
      @freeze = freeze
    end

    def read_document
      major = next_byte
      minor = next_byte
      if major != Marshal::MAJOR_VERSION || minor > Marshal::MINOR_VERSION
        raise TypeError, "incompatible marshal file format (can't be read)"
      end
      read
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
      raise ArgumentError, "marshal data too short" if @at + count > @bytes.length
      held = @bytes[@at, count]
      @at += count
      held.pack("C*").force_encoding Encoding::BINARY
    end

    # Put an object in the table links refer to, as one still being built.
    def remember(object)
      @objects << object
      @building[object.__id__] = true
      @objects.length - 1
    end

    # A value is finished: frozen when asked for, with a String swapped for
    # the one frozen copy every equal String shares, and then handed to the
    # proc, whose answer is what the value is read as.
    def leave(value, at = nil)
      @building.delete value.__id__
      if @freeze
        frozen = Marshal.__frozen__(value)
        @objects[at] = frozen if !at.nil? && !frozen.equal?(value)
        value = frozen
      end
      @handler.nil? ? value : @handler.call(value)
    end

    def read
      read_object false, [], false
    end

    # `variables` says instance variables follow the value, `extensions` are
    # the modules it is to be extended with, and `partial` says the caller
    # finishes it rather than this.
    def read_object(variables, extensions, partial)
      case next_byte.chr
      when "0" then leave nil
      when "T" then leave true
      when "F" then leave false
      when "i" then leave read_long
      when ":" then leave read_symbol_body(variables)
      when ";" then linked_symbol
      when "@" then read_link
      when "I" then read_object true, extensions, partial
      when "e" then read_extended variables, extensions, partial
      when "C" then read_user_class variables, extensions, partial
      when '"' then read_string variables, extensions, partial
      when "/" then read_regexp variables, extensions, partial
      when "[" then read_array variables, extensions, partial
      when "{" then read_hash false, variables, extensions, partial
      when "}" then read_hash true, variables, extensions, partial
      when "f"
        at = remember read_float
        finish @objects[at], at, extensions, partial
      when "l"
        at = remember read_bignum
        finish @objects[at], at, extensions, partial
      when "c" then read_named Class, "class", variables
      when "m" then read_named Module, "module", variables
      when "M" then read_named Module, "module", variables
      when "o" then read_instance variables, extensions, partial
      when "S" then read_struct variables, extensions, partial
      when "U" then read_user_marshal extensions, partial
      when "d" then read_data extensions, partial
      when "u" then read_user_defined variables, extensions, partial
      else raise ArgumentError, "dump format error"
      end
    end

    # Extend a value with the modules written before it, then finish it
    # unless the one reading it finishes it. A value built by `_load` or
    # `marshal_load` is frozen and handed to the proc like any other, and
    # stays among those being built, so a link to it later is not handed to
    # the proc again.
    def finish(value, at, extensions, partial, stays_building = false)
      extensions.reverse_each { |held| value.extend held }
      return value if partial
      return leave(value, at) unless stays_building
      value = Marshal.__frozen__(value) if @freeze
      @handler.nil? ? value : @handler.call(value)
    end

    def linked_symbol
      at = read_long
      raise ArgumentError, "bad symbol" if at >= @symbols.length
      @symbols[at]
    end

    def read_link
      at = read_long
      raise ArgumentError, "dump format error (unlinked)" if at >= @objects.length
      value = @objects[at]
      return value if @building.key? value.__id__
      @handler.nil? ? value : @handler.call(value)
    end

    # The name of an instance variable or of a member, which is a Symbol or a
    # link to one and is not a value of its own.
    def read_name
      case next_byte.chr
      when ":" then read_symbol_body false
      when ";" then linked_symbol
      when "I"
        raise ArgumentError, "dump format error (symlink with encoding)" unless next_byte.chr == ":"
        read_symbol_body true
      else raise ArgumentError, "dump format error for symbol(0x#{@bytes[@at - 1].to_s(16)})"
      end
    end

    def read_symbol_body(variables)
      spelled = read_bytes read_long
      at = @symbols.length
      @symbols << nil
      if variables
        read_long.times do
          name = read_name
          value = read
          spelled = Marshal.__encoded__(spelled, name, value)
        end
      end
      spelled = spelled.dup.force_encoding(Encoding::US_ASCII) if spelled.ascii_only?
      @symbols[at] = spelled.to_sym
    end

    # The variables written after a value: its encoding for text, and its own
    # instance variables.
    def read_variables(made)
      read_long.times do
        name = read_name
        value = read
        if made.is_a?(String) && (name == :E || name == :encoding)
          made.force_encoding Marshal.__encoded__("".b, name, value).encoding
        elsif name != :E && name != :encoding
          Marshal.__set_variable__ made, name, value
        end
      end
    end

    def read_string(variables, extensions, partial)
      made = read_bytes read_long
      at = remember made
      read_variables made if variables
      finish made, at, extensions, partial
    end

    # A Regexp is built once its source has its encoding, which is written
    # after it among its variables.
    def read_regexp(variables, extensions, partial)
      source = read_bytes read_long
      options = next_byte
      at = remember nil
      held = []
      if variables
        read_long.times do
          name = read_name
          value = read
          if name == :E || name == :encoding
            source = Marshal.__encoded__(source, name, value)
          else
            held << [name, value]
          end
        end
      end
      made = Regexp.new(source, options)
      @objects[at] = made
      held.each { |name, value| Marshal.__set_variable__ made, name, value }
      finish made, at, extensions, partial
    end

    def read_array(variables, extensions, partial)
      made = []
      at = remember made
      read_long.times { made << read }
      read_variables made if variables
      finish made, at, extensions, partial
    end

    def read_hash(with_default, variables, extensions, partial)
      made = {}
      at = remember made
      read_long.times do
        key = read
        made[key] = read
      end
      made.default = read if with_default
      read_variables made if variables
      finish made, at, extensions, partial
    end

    def read_extended(variables, extensions, partial)
      named = read_name.to_s
      held = Marshal.__path_to_class__(named)
      raise ArgumentError, "#{named} does not refer to module" unless held.instance_of? Module
      read_object variables, extensions + [held], partial
    end

    # A String, Array, Hash or Regexp of a subclass, built as the subclass
    # from the value read. `Hash` itself stands for a Hash that compares its
    # keys by identity.
    def read_user_class(variables, extensions, partial)
      held_class = Marshal.__path_to_class__(read_name.to_s)
      at = @objects.length
      inner = read_object(variables, [], true)
      made = Marshal.__as_subclass__(held_class, inner)
      @objects[at] = made unless at >= @objects.length
      finish made, at, extensions, partial
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

    # A class or a module, found by the name it was written under.
    def read_named(kind, word, variables)
      named = read_bytes read_long
      if variables
        read_long.times do
          name = read_name
          value = read
          named = Marshal.__encoded__(named, name, value)
        end
      end
      named = named.dup.force_encoding(Encoding::UTF_8)
      held = Marshal.__path_to_class__(named)
      refers = kind.equal?(Class) ? held.is_a?(Class) : held.instance_of?(Module)
      raise ArgumentError, "#{named} does not refer to #{word}" unless refers
      at = remember held
      @building.delete held.__id__
      @handler.nil? ? held : leave(held, at)
    end

    def read_instance(variables, extensions, partial)
      named = read_name.to_s
      held_class = Marshal.__path_to_class__(named)
      raise ArgumentError, "#{named} does not refer to class" unless held_class.is_a? Class
      return read_range(held_class, extensions, partial) if held_class <= Range
      # A class whose instances are values of their own kind is never written
      # as a plain object.
      if Marshal.__value_classes__.any? { |kind| held_class <= kind }
        raise ArgumentError, "dump format error"
      end
      made = held_class.allocate
      at = remember made
      read_long.times do
        name = read_name
        value = read
        if made.is_a? Exception
          apply_exception_variable made, name, value
        else
          Marshal.__set_variable__ made, name, value
        end
      end
      finish made, at, extensions, partial
    end

    # A Range is written as its three parts, and is built from them once they
    # are read, since a Range cannot change after it is made.
    def read_range(held_class, extensions, partial)
      at = remember nil
      parts = {}
      read_long.times do
        name = read_name
        parts[name] = read
      end
      made = held_class.new(parts[:begin], parts[:end], parts[:excl])
      @objects[at] = made
      finish made, at, extensions, partial
    end

    # An Exception's message, backtrace and cause are written under names of
    # their own rather than as instance variables.
    def apply_exception_variable(made, name, value)
      case name
      when :mesg then made.__restore_message__ value
      when :bt then made.set_backtrace value unless value.nil?
      when :cause then made.__restore_cause__ value
      when :bt_locations, :private_call? then nil
      when :name, :args then made.__restore_attribute__ name, value
      else Marshal.__set_variable__ made, name, value
      end
    end

    # A Struct or a Data, whose members must be the ones its class has.
    def read_struct(variables, extensions, partial)
      named = read_name.to_s
      held_class = Marshal.__path_to_class__(named)
      unless held_class <= Struct || held_class <= Data
        raise TypeError, "class #{named} not a struct"
      end
      at = remember nil
      values = {}
      read_long.times do
        name = read_name
        values[name] = read
      end
      members = held_class.members
      unless values.keys == members
        raise TypeError, "struct #{named} not compatible (#{values.keys.inspect} for #{members.inspect})"
      end
      made = if held_class <= Data
        held_class.new(**values)
      else
        built = held_class.allocate
        values.each { |name, value| built[name] = value }
        built
      end
      @objects[at] = made
      read_variables made if variables
      finish made, at, extensions, partial
    end

    def read_user_marshal(extensions, partial)
      named = read_name.to_s
      held_class = Marshal.__path_to_class__(named)
      # A Rational or a Complex has no allocator, and is built from the two
      # numbers written for it.
      if held_class <= Rational || held_class <= Complex
        at = remember nil
        parts = read
        made = held_class <= Rational ? Rational(*parts) : Complex(*parts)
        @objects[at] = made
        return finish(made, at, extensions, partial)
      end
      made = held_class.allocate
      unless Marshal.__responds__(made, :marshal_load)
        raise TypeError, "instance of #{named} needs to have method 'marshal_load'"
      end
      at = remember made
      made.__send__ :marshal_load, read
      finish made, at, extensions, partial, true
    end

    # An object whose class keeps it in a form of its own, put back by
    # `_load_data` from what `_dump_data` wrote.
    def read_data(extensions, partial)
      named = read_name.to_s
      held_class = Marshal.__path_to_class__(named)
      # Only a class whose objects the interpreter holds in a form of its own
      # is written this way.
      unless [Dir, IO, Proc, Method, UnboundMethod, Thread, Mutex, Random, Binding].any? { |kind| held_class <= kind }
        raise ArgumentError, "dump format error"
      end
      made = held_class.allocate
      unless Marshal.__responds__(made, :_load_data)
        raise TypeError, "class #{named} needs to have instance method '_load_data'"
      end
      at = remember made
      made.__send__ :_load_data, read
      finish made, at, extensions, partial
    end

    def read_user_defined(variables, extensions, partial)
      named = read_name.to_s
      # Ruby writes a NameError's message as an object that formats it when
      # asked, and reads it back as the text.
      held_class = named == "NameError::message" ? nil : Marshal.__path_to_class__(named)
      data = read_bytes read_long
      if variables
        read_long.times do
          name = read_name
          value = read
          # A Time's offset and zone travel beside its bytes under names
          # that are not instance variables, and its `_load` reads them.
          if [:offset, :zone, :nano_num, :nano_den, :submicro].include? name
            data.instance_variable_set :"@__marshal_#{name}", value
          elsif name == :E || name == :encoding
            data = Marshal.__encoded__(data, name, value)
          else
            data.instance_variable_set name, value
          end
        end
      end
      if held_class.nil?
        at = remember data
        return finish(data, at, extensions, partial, true)
      end
      unless held_class.respond_to? :_load, true
        raise TypeError, "class #{named} needs to have method '_load'"
      end
      made = held_class.__send__ :_load, data
      at = remember made
      finish made, at, extensions, partial, true
    end
  end
end
"##;
