pub(super) const SOURCE: &str = r##"
class Time
  include Comparable

  MICROSECONDS_IN_SECOND = 1000000
  NANOSECONDS_IN_SECOND = 1000000000
  MONTH_NAMES = %w[jan feb mar apr may jun jul aug sep oct nov dec]

  def self.now(**options)
    counted = Time.__now__
    made = from_exact(counted[0].to_r + Rational(counted[1], NANOSECONDS_IN_SECOND), false, nil)
    zone = options[:in]
    return made if zone.nil?
    return made.utc if names_utc?(zone)
    made.localtime zone
  end

  def self.new(*args, **options)
    return now(**options) if args.empty?
    if args.size == 1 && (args[0].is_a?(String) || args[0].respond_to?(:to_str))
      return from_written(args[0], options)
    end
    if args.size > 6 && !args[6].nil? && !options[:in].nil?
      raise ArgumentError, "timezone argument given as positional and keyword arguments"
    end
    zone = args.size > 6 ? args[6] : options[:in]
    fields = args[0, 6]
    return from_exact(calendar_seconds(fields, false), false, nil) if zone.nil?
    # A zone may be an object that converts between local and UTC readings,
    # which is what a timezone library hands over.
    zone = zone_named(self, zone)
    if zone.respond_to?(:local_to_utc)
      reading = calendar_seconds(fields, true)
      answered = zone.local_to_utc(from_exact(reading, true, nil))
      # A Time stands at the instant it names. Anything else names only the
      # whole seconds it reads as, so its own zone and offset are left out.
      counted = if answered.is_a?(Time)
        answered.to_r
      elsif answered.respond_to?(:to_i)
        answered.to_i
      else
        raise TypeError, "can't convert #{answered.class} into an exact number"
      end
      offset = (reading - counted).to_i
      raise ArgumentError, "utc_offset out of range" if offset.abs >= 86400
      made = from_exact(counted, false, offset)
      made.instance_variable_set(:@zone_object, zone)
      return made
    end
    return from_exact(calendar_seconds(fields, true), true, nil) if names_utc?(zone)
    offset = offset_seconds(zone)
    from_exact(calendar_seconds(fields, true) - offset, false, offset)
  end

  def self.at(seconds, *rest, **options)
    if rest.size > 2
      raise ArgumentError, "wrong number of arguments (given #{1 + rest.size}, expected 1..3)"
    end
    extra = rest[0]
    made = if rest.empty?
      if seconds.is_a? Time
        from_exact(seconds.to_r, seconds.utc?, nil)
      else
        from_exact(exact_number(seconds), false, nil)
      end
    else
      # A second count alongside a Time has nothing to add to, which is what
      # Ruby refuses.
      if seconds.is_a? Time
        raise TypeError, "can't convert Time into an exact number"
      end
      from_exact(
        exact_number(seconds) + exact_number(extra) / fraction_of_second(rest[1], rest.size > 1),
        false,
        nil
      )
    end
    zone = options[:in]
    return made if zone.nil?
    return made.utc if names_utc?(zone)
    made.localtime zone
  end

  def self.utc(*args)
    from_exact(calendar_seconds(args, true), true, nil)
  end

  def self.gm(*args)
    utc(*args)
  end

  def self.local(*args)
    from_exact(calendar_seconds(args, false), false, nil)
  end

  def self.mktime(*args)
    local(*args)
  end

  # A time built from an exact count of seconds since the epoch.
  def self.from_exact(total, utc, offset)
    exact = total.to_r
    whole = exact.floor
    made = allocate
    made.instance_variable_set(:@seconds, whole)
    made.instance_variable_set(:@fraction, exact - whole)
    made.instance_variable_set(:@utc, utc)
    made.instance_variable_set(:@offset, offset)
    # A time reading in the zone the program runs in reads it now, since that
    # zone may change before a field is asked for.
    made.send(:__read_calendar__) if !utc && offset.nil?
    made
  end

  # The seconds a calendar argument list stands for, where the last argument
  # is a count of microseconds.
  def self.calendar_seconds(args, utc)
    # Ten arguments name the parts in the order the C library writes them,
    # seconds first and the year sixth. The rest are the day of the week, the
    # day of the year, the daylight-saving flag, and the zone, none of which
    # take part in the count.
    daylight = nil
    if args.size == 10
      daylight = args[8] if args[8] == true || args[8] == false
      args = [args[5], args[4], args[3], args[2], args[1], args[0]]
    elsif args.empty? || args.size > 7
      raise ArgumentError,
            "wrong number of arguments (given #{args.size}, expected 1..7)"
    end
    raise TypeError, "no implicit conversion from nil to integer" if args[0].nil?
    year = time_part(args[0], "year")
    month = month_number(args[1])
    day = args[2].nil? ? 1 : time_part(args[2], "day")
    hour = args[3].nil? ? 0 : time_part(args[3], "hour")
    minute = args[4].nil? ? 0 : time_part(args[4], "min")
    second = args[5].nil? ? 0 : time_second(args[5])
    named_micro = args.size > 6 && args[6].is_a?(Numeric)
    micro = named_micro ? args[6] : 0
    raise ArgumentError, "mon out of range" unless (1..12).cover?(month)
    raise ArgumentError, "mday out of range" unless (1..31).cover?(day)
    raise ArgumentError, "hour out of range" unless (0..24).cover?(hour)
    raise ArgumentError, "min out of range" unless (0..59).cover?(minute)
    raise ArgumentError, "argument out of range" if second < 0 || micro < 0
    raise ArgumentError, "sec out of range" if second >= 61
    raise ArgumentError, "subsecx out of range" if micro >= MICROSECONDS_IN_SECOND
    whole = Time.__assemble__(year, month, day, hour, minute, second.to_i, utc, daylight)
    # A count of microseconds given outright stands for the whole fraction,
    # so a fraction carried by the seconds is left out.
    fraction = named_micro ? 0 : second.to_r - second.to_i
    whole.to_r + fraction + micro.to_r / MICROSECONDS_IN_SECOND
  end

  # One whole part of a calendar time. A numeral spelled out reads as base
  # ten, and anything else has to read as an Integer.
  def self.time_part(held, name)
    return held if held.is_a?(Integer)
    return held.to_i if held.is_a?(Numeric)
    if held.is_a?(String)
      unless held =~ /\A\s*[+-]?\d+\s*\z/
        raise ArgumentError, "argument out of range"
      end
      return held.to_i(10)
    end
    unless held.respond_to?(:to_int)
      raise TypeError, "no implicit conversion of #{held.class} into Integer"
    end
    read = held.to_int
    unless read.is_a?(Integer)
      raise TypeError, "can't convert #{held.class} into Integer"
    end
    read
  end

  # The seconds part, which may carry a fraction.
  def self.time_second(held)
    return held if held.is_a?(Numeric)
    return time_part(held, "sec") unless held.is_a?(String)
    return held.to_i(10) if held =~ /\A\s*[+-]?\d+\s*\z/
    raise ArgumentError, "argument out of range"
  end

  def self.month_number(named)
    return 1 if named.nil?
    return named.to_i if named.is_a?(Numeric)
    text = if named.is_a?(String)
             named
           elsif named.respond_to?(:to_str)
             named.to_str
           elsif named.respond_to?(:to_int)
             return time_part(named, "mon")
           else
             named.to_s
           end
    return text.to_i(10) if text =~ /\A\s*[+-]?\d+\s*\z/
    found = MONTH_NAMES.index(text.downcase[0, 3])
    raise ArgumentError, "mon out of range" if found.nil?
    found + 1
  end

  # The seconds an offset stands for, given either as a count of seconds or
  # as the `"+05:00"` spelling.
  # Whether a zone names UTC rather than an offset from it.
  def self.names_utc?(zone)
    return false if zone.nil? || zone.is_a?(Numeric)
    return false if zone.respond_to? :local_to_utc
    text = zone.to_s
    # A zone written as a negative zero offset is UTC itself rather than a
    # place that happens to sit on it.
    text == "UTC" || text == "Z" || text == "-00:00" || text == "-0000"
  end

  def self.offset_seconds(offset)
    return in_range(offset) if offset.is_a?(Integer)
    return in_range(offset.to_r) if offset.is_a?(Numeric)
    # A number or a name of its own is asked for, in that order, which is
    # what Ruby asks an object standing in for either.
    if !offset.is_a?(String) && offset.respond_to?(:to_int)
      return in_range(offset.to_int)
    end
    unless offset.is_a?(String) || offset.respond_to?(:to_str)
      raise TypeError, "can't convert #{offset.class} into an exact number"
    end
    offset = offset.to_str unless offset.is_a?(String)
    text = offset.to_s
    # An offset in an encoding that spells ASCII in wider units carries the
    # zero bytes between its characters, which Ruby refuses outright.
    raise ArgumentError, "string contains null byte" unless text.encoding.ascii_compatible?
    return 0 if text == "UTC" || text == "Z"
    letter = military_offset text
    return letter unless letter.nil?
    unless text =~ /\A([+-])(\d\d)(?::?(\d\d))?(?::?(\d\d))?\z/
      raise ArgumentError, "\"+HH:MM\", \"-HH:MM\", \"UTC\" or \"A\"..\"I\",\"K\"..\"Z\" expected for utc_offset: #{offset}"
    end
    hours = $2.to_i
    minutes = $3.nil? ? 0 : $3.to_i
    seconds = $4.nil? ? 0 : $4.to_i
    raise ArgumentError, "utc_offset out of range" if hours > 23
    if minutes > 59 || seconds > 59
      raise ArgumentError, "\"+HH:MM\", \"-HH:MM\", \"UTC\" or \"A\"..\"I\",\"K\"..\"Z\" expected for utc_offset: #{offset}"
    end
    counted = hours * 3600 + minutes * 60 + seconds
    $1 == "-" ? -counted : counted
  end

  # The time a written date names, in the shape Ruby reads: a year on its own,
  # or a whole date and time with an offset after it.
  def self.from_written(written, options)
    written = written.to_str unless written.is_a?(String)
    unless written.encoding.ascii_compatible?
      raise ArgumentError, "time string should have ASCII compatible encoding"
    end
    shape = /\A(\d{4,})(?:-(\d\d)-(\d\d)[ T](\d\d):(\d\d):(\d\d)(?:\.(\d+))?(?:[ ]?([+-][\d:]+|Z|UTC))?)?\z/
    held = shape.match written
    raise ArgumentError, "can't parse: #{written.inspect}" if held.nil?
    fields = [held[1].to_i, (held[2] || 1).to_i, (held[3] || 1).to_i,
              (held[4] || 0).to_i, (held[5] || 0).to_i, (held[6] || 0).to_i]
    fraction = written_fraction held[7], options
    fields[5] = fields[5] + fraction
    zone = held[8].nil? ? options[:in] : held[8]
    return new(*fields) if zone.nil?
    new(*fields, zone)
  end

  # The fraction of a second the digits after the point name, cut down to the
  # count of places asked for.
  def self.written_fraction(digits, options)
    return 0 if digits.nil?
    places = options.key?(:precision) ? options[:precision] : 9
    unless places.nil?
      places = places.to_int if !places.is_a?(Integer) && places.respond_to?(:to_int)
      places = places.to_i if places.is_a?(Numeric)
      unless places.is_a?(Integer)
        raise TypeError, "no implicit conversion of #{places.class} into Integer"
      end
      digits = digits[0, places].to_s if places >= 0
    end
    return 0 if digits.empty?
    Rational(digits.to_i, 10 ** digits.length)
  end

  # How many of the units a third argument names make up a second.
  def self.fraction_of_second(named, given)
    return MICROSECONDS_IN_SECOND unless given
    case named
    when :nanosecond, :nsec then NANOSECONDS_IN_SECOND
    when :microsecond, :usec then MICROSECONDS_IN_SECOND
    when :millisecond then 1000
    else raise ArgumentError, "unexpected unit: #{named}"
    end
  end

  # A count of seconds read as an exact number. A number stands for itself,
  # and anything else has to name a whole number before its fraction is read.
  def self.exact_number(value)
    return value.to_r if value.is_a?(Numeric)
    unless value.respond_to?(:to_int)
      named = value.nil? ? "nil" : value.class.to_s
      raise TypeError, "can't convert #{named} into an exact number"
    end
    return value.to_r if value.respond_to?(:to_r)
    value.to_int.to_r
  end

  # An offset stands no further from UTC than a day, which is what Ruby
  # refuses a wider one against.
  def self.in_range(counted)
    raise ArgumentError, "utc_offset out of range" if counted.abs >= 86400
    counted
  end

  # The offset one of the military zone letters names. "A" through "M" count
  # east of Greenwich, leaving "J" out, and "N" through "Y" count west.
  def self.military_offset(text)
    return nil unless text.length == 1
    letter = text.upcase
    return nil unless ("A".."Z").cover?(letter) && letter != "J"
    place = letter.ord - "A".ord
    return 3600 * (place + 1) if letter <= "I"
    return 3600 * place if letter <= "M"
    return 0 if letter == "Z"
    -3600 * (letter.ord - "N".ord + 1)
  end

  # Whether a zone converts between its own readings and UTC rather than
  # naming a fixed offset.
  def self.zone_object?(zone)
    !zone.nil? && zone.respond_to?(:utc_to_local)
  end

  # The zone a name stands for. A class carrying `find_timezone` is asked for
  # a name no offset reads as, which is how a timezone library is reached.
  def self.zone_named(holder, offset)
    return offset if offset.nil? || zone_object?(offset)
    # Only a name written as text reaches a timezone library. A symbol, an
    # array, or anything else is read as a number and refused as one.
    return offset unless offset.is_a?(String)
    return offset unless holder.respond_to?(:find_timezone)
    begin
      offset_seconds offset
      offset
    rescue ArgumentError, TypeError
      holder.find_timezone offset
    end
  end

  # Read the calendar fields now and keep them, so a later change to the zone
  # the program runs in does not reach a time already read.
  def __read_calendar__
    calendar
    nil
  end
  private :__read_calendar__

  # The calendar fields this time stands for, read once and kept.
  def calendar
    return @calendar unless @calendar.nil?
    @calendar = if @offset.nil?
      Time.__breakdown__(@seconds, @utc)
    else
      Time.__breakdown__((@seconds + @offset).floor, true)
    end
  end
  private :calendar

  def year
    calendar[0]
  end

  def mon
    calendar[1]
  end

  def month
    calendar[1]
  end

  def day
    calendar[2]
  end

  def mday
    calendar[2]
  end

  def hour
    calendar[3]
  end

  def min
    calendar[4]
  end

  def sec
    calendar[5]
  end

  def wday
    calendar[6]
  end

  def yday
    calendar[7]
  end

  def isdst
    @offset.nil? && !@utc && calendar[8]
  end

  def dst?
    isdst
  end

  def utc_offset
    return @offset unless @offset.nil?
    return 0 if @utc
    calendar[9]
  end

  def gmt_offset
    utc_offset
  end

  def gmtoff
    utc_offset
  end

  # The name the zone goes by, which is written in ASCII whatever the text
  # of the program is written in.
  def zone
    return @zone_object unless @zone_object.nil?
    return @zone_name unless @zone_name.nil?
    return nil unless @offset.nil?
    return "UTC".force_encoding(Encoding::US_ASCII) if @utc
    named = calendar[10]
    named.nil? ? nil : named.force_encoding(Encoding::US_ASCII)
  end

  def sunday?
    wday == 0
  end

  def monday?
    wday == 1
  end

  def tuesday?
    wday == 2
  end

  def wednesday?
    wday == 3
  end

  def thursday?
    wday == 4
  end

  def friday?
    wday == 5
  end

  def saturday?
    wday == 6
  end

  def to_i
    @seconds
  end

  def tv_sec
    @seconds
  end

  def to_f
    to_r.to_f
  end

  def to_r
    @seconds.to_r + @fraction
  end

  def subsec
    @fraction == 0 ? 0 : @fraction
  end

  def usec
    (@fraction * MICROSECONDS_IN_SECOND).to_i
  end

  def tv_usec
    usec
  end

  def nsec
    (@fraction * NANOSECONDS_IN_SECOND).to_i
  end

  def tv_nsec
    nsec
  end

  def utc?
    @utc
  end
  # The time the eight bytes Marshal writes stand for. The newer form packs
  # the date and clock into two words, and the older one holds a UNIX
  # timestamp and the microseconds beside it.
  # The variables a program set on the Time, without the ones the Time keeps
  # its own state in.
  def instance_variables
    Kernel.instance_method(:instance_variables).bind_call(self) - Marshal::TIME_STATE
  end

  def self._load(written)
    high, low = written.dup.force_encoding(Encoding::BINARY).unpack "VV"
    if (high >> 31) & 1 == 0
      return Time.at(high, low)
    end
    in_utc = (high >> 30) & 1 == 1
    built = Time.utc((((high >> 14) & 0xffff) + 1900),
                     ((high >> 10) & 0xf) + 1,
                     (high >> 5) & 0x1f,
                     high & 0x1f,
                     (low >> 26) & 0x3f,
                     (low >> 20) & 0x3f,
                     low & 0xfffff)
    nano_num = written.instance_variable_get :@__marshal_nano_num
    nano_den = written.instance_variable_get :@__marshal_nano_den
    submicro = written.instance_variable_get :@__marshal_submicro
    nano = if !nano_num.nil? && !nano_den.nil?
      Rational(nano_num, nano_den)
    elsif !submicro.nil?
      high_digits, low_digits = submicro.bytes
      (high_digits >> 4) * 100 + (high_digits & 0xf) * 10 + (low_digits.to_i >> 4)
    end
    built += Rational(nano, 1_000_000_000) unless nano.nil? || nano.zero?
    named = written.instance_variable_get :@__marshal_zone
    offset = written.instance_variable_get :@__marshal_offset
    made = if !named.nil? && respond_to?(:find_timezone)
      built.getlocal(find_timezone(named))
    elsif in_utc
      built
    elsif !offset.nil?
      shifted = built.getlocal(offset)
      shifted.instance_variable_set :@zone_name, named unless named.nil?
      shifted
    else
      built.localtime
    end
    # What the program set on the Time came along with the bytes.
    written.instance_variables.each do |name|
      next if name.to_s.start_with? "@__"
      made.instance_variable_set name, written.instance_variable_get(name)
    end
    made
  end
  private_class_method :_load
"##;
