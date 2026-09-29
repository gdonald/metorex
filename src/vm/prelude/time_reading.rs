pub(super) const SOURCE: &str = r##"
  # The eight bytes Marshal writes a Time as: the date and hour packed into
  # one word and the rest of the clock into another, always read in UTC, with
  # a flag saying whether the time itself stands in UTC.
  def _dump(_limit = 0)
    held = getutc
    high = 1 << 31 |
           (gmt? ? 1 : 0) << 30 |
           (held.year - 1900) << 14 |
           (held.mon - 1) << 10 |
           held.mday << 5 |
           held.hour
    low = held.min << 26 | held.sec << 20 | held.usec
    # The offset and the zone travel beside the bytes, which is where Marshal
    # writes them.
    [high, low].pack "VV"
  end
  private :_dump


  def gmt?
    @utc
  end

  def utc
    return self if @utc && @offset.nil?
    @utc = true
    @offset = nil
    @calendar = nil
    self
  end

  def gmtime
    utc
  end

  def getutc
    Time.from_exact(to_r, true, nil)
  end

  def getgm
    getutc
  end

  def localtime(offset = nil)
    offset = Time.zone_named(self.class, offset)
    return utc if Time.names_utc? offset
    if Time.zone_object? offset
      @utc = false
      @offset = __zone_offset__ offset
      @zone_object = offset
      @calendar = nil
      return self
    end
    wanted = offset.nil? ? nil : Time.offset_seconds(offset)
    # A time already reading in the zone asked for is left alone, so a frozen
    # one is not refused for a change it does not need.
    return self if !@utc && @offset == wanted
    @utc = false
    @offset = wanted
    @calendar = nil
    # The zone the program is running in is read now rather than the next
    # time a field is asked for, since that zone may change in between.
    __read_calendar__ if wanted.nil?
    self
  end

  def getlocal(offset = nil)
    offset = Time.zone_named(self.class, offset)
    return getutc if Time.names_utc? offset
    if Time.zone_object? offset
      made = Time.from_exact(to_r, false, __zone_offset__(offset))
      made.instance_variable_set(:@zone_object, offset)
      return made
    end
    Time.from_exact(to_r, false, offset.nil? ? nil : Time.offset_seconds(offset))
  end

  # How far a zone of its own stands from UTC at this moment, read by asking
  # the zone what local reading this one has. The answer is read for the
  # calendar fields it names rather than the instant it says it stands at, so
  # a zone or an offset it carries of its own is left out of the count.
  def __zone_offset__(zone)
    reading = zone.utc_to_local(Time.from_exact(to_r, true, nil))
    counted = if reading.is_a?(Integer)
      reading
    elsif reading.respond_to?(:year) && reading.respond_to?(:mday)
      Time.utc(reading.year, reading.mon, reading.mday, reading.hour, reading.min, reading.sec).to_i
    else
      reading.to_r.floor
    end
    found = counted - to_r.floor
    raise ArgumentError, "utc_offset out of range" if found.abs >= 86400
    found
  end
  private :__zone_offset__

  def +(other)
    raise TypeError, "time + time?" if other.is_a?(Time)
    shifted(to_r + exact_seconds(other))
  end

  def -(other)
    return (to_r - other.to_r).to_f if other.is_a?(Time)
    shifted(to_r - exact_seconds(other))
  end

  # A time at another point, reading in the same zone this one does.
  def shifted(total)
    made = Time.from_exact(total, @utc, @offset)
    made.instance_variable_set(:@zone_object, @zone_object) unless @zone_object.nil?
    # A time read in the zone the program was running in keeps that zone's
    # name, which a later change to the zone does not reach.
    if @offset.nil? && !@utc && !@calendar.nil?
      made.instance_variable_set(:@zone_name, @calendar[10])
    end
    made
  end
  private :shifted

  # The seconds an argument to `+` or `-` stands for. A String names no
  # number of seconds however much it looks like one.
  def exact_seconds(other)
    raise TypeError, "can't convert String into an exact number" if other.is_a?(String)
    return other.to_r if other.is_a?(Numeric)
    unless other.respond_to?(:to_r)
      raise TypeError, "can't convert #{other.class} into an exact number"
    end
    other.to_r
  end
  private :exact_seconds

  # Two times order by the moment each one names. Anything else is asked to
  # order itself against this time, and the answer is turned around.
  def <=>(other)
    return to_r <=> other.to_r if other.is_a?(Time)
    return nil unless other.respond_to?(:<=>)
    answered = other <=> self
    return nil if answered.nil?
    return -1 if answered > 0
    return 1 if answered < 0
    0
  end

  def ==(other)
    other.is_a?(Time) && to_r == other.to_r
  end

  def eql?(other)
    other.is_a?(Time) && to_r == other.to_r
  end

  def hash
    to_r.hash
  end

  def floor(digits = 0)
    scale = 10 ** digits
    shifted(Rational((to_r * scale).floor, scale))
  end

  def ceil(digits = 0)
    scale = 10 ** digits
    shifted(Rational((to_r * scale).ceil, scale))
  end

  def round(digits = 0)
    scale = 10 ** digits
    shifted(Rational((to_r * scale).round, scale))
  end

  def to_a
    [sec, min, hour, mday, mon, year, wday, yday, isdst, zone]
  end

  def deconstruct_keys(keys)
    all = {
      year: year, month: month, day: day, yday: yday, wday: wday,
      hour: hour, min: min, sec: sec, subsec: subsec, dst: dst?, zone: zone
    }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end

  # The text a template names, with each directive filled in. Every directive
  # Ruby writes is read here rather than handed to the C library, so the flags
  # and widths Ruby adds are answered the same way everywhere.
  def strftime(template)
    template = template.to_str unless template.is_a?(String)
    written = +""
    at = 0
    while at < template.length
      held = template[at]
      unless held == "%"
        written << held
        at += 1
        next
      end
      read = __read_directive__(template, at)
      if read.nil?
        written << held
        at += 1
        next
      end
      written << read[0]
      at = read[1]
    end
    written
  end

  # One directive, read from the template at `at`. Answers the text it stands
  # for and where the template carries on, or nil where it names none.
  def __read_directive__(template, at)
    index = at + 1
    flags = +""
    while index < template.length && "-_0^#".include?(template[index])
      flags << template[index]
      index += 1
    end
    width = +""
    while index < template.length && template[index] =~ /\d/
      width << template[index]
      index += 1
    end
    colons = 0
    while index < template.length && template[index] == ":"
      colons += 1
      index += 1
    end
    return nil if index >= template.length
    letter = template[index]
    return nil if colons > 0 && letter != "z"
    width = width.empty? ? 0 : width.to_i
    piece = __directive_text__(letter, flags, width, colons)
    return nil if piece.nil?
    piece = piece.upcase if flags.include?("^")
    piece = piece.swapcase if flags.include?("#")
    # A width names the least room a directive takes, whether it wrote a
    # number or a name.
    if width > piece.length && !flags.include?("-") && !__counts_its_own__(letter)
      piece = piece.rjust(width, __padding_of__(flags, " "))
    end
    [piece, index + 1]
  end
  private :__read_directive__

  # The character a directive pads with. The last of the two flags that name
  # one has the say, and without either the directive's own stands.
  def __padding_of__(flags, held)
    flags.each_char do |flag|
      held = " " if flag == "_"
      held = "0" if flag == "0"
    end
    held
  end
  private :__padding_of__

  # Whether a directive fills its own width, which every number does.
  def __counts_its_own__(letter)
    "YCymdejHkIlMSsuwUWVGgzNL".include?(letter)
  end
  private :__counts_its_own__

  WRITTEN_DAY_NAMES = %w[Sunday Monday Tuesday Wednesday Thursday Friday Saturday]
  WRITTEN_MONTH_NAMES = %w[January February March April May June July August
                           September October November December]
  private_constant :WRITTEN_DAY_NAMES
  private_constant :WRITTEN_MONTH_NAMES

  # The text one directive stands for.
  def __directive_text__(letter, flags, width, colons)
    case letter
    when "%" then return "%"
    when "n" then return "\n"
    when "t" then return "\t"
    when "z" then return __offset_text__(flags, width, colons)
    when "Z"
      unless @zone_object.nil?
        return @zone_object.abbr(self).to_s if @zone_object.respond_to?(:abbr)
        return @zone_object.to_s
      end
      return zone.to_s
    when "a" then return WRITTEN_DAY_NAMES[wday][0, 3]
    when "A" then return WRITTEN_DAY_NAMES[wday]
    when "b", "h" then return WRITTEN_MONTH_NAMES[mon - 1][0, 3]
    when "B" then return WRITTEN_MONTH_NAMES[mon - 1]
    when "p" then return hour < 12 ? "AM" : "PM"
    when "P" then return hour < 12 ? "am" : "pm"
    when "c" then return strftime("%a %b %e %H:%M:%S %Y")
    when "x", "D" then return strftime("%m/%d/%y")
    when "X", "T" then return strftime("%H:%M:%S")
    when "F" then return strftime("%Y-%m-%d")
    when "R" then return strftime("%H:%M")
    when "r" then return strftime("%I:%M:%S %p")
    when "v" then return strftime("%e-%^b-%Y")
    when "+" then return strftime("%a %b %e %H:%M:%S %Z %Y")
    when "N" then return __fraction_text__(width == 0 ? 9 : width)
    when "L" then return __fraction_text__(width == 0 ? 3 : width)
    end
    counted, pad, places = __directive_number__(letter)
    return nil if counted.nil?
    __padded_number__(counted, flags, width, pad, places)
  end
  private :__directive_text__

  # The number a directive stands for, the character it pads with, and how
  # many places it takes when nothing else is asked for.
  def __directive_number__(letter)
    case letter
    when "Y" then [year, "0", 4]
    when "C" then [year / 100, "0", 2]
    when "y" then [year % 100, "0", 2]
    when "m" then [mon, "0", 2]
    when "d" then [mday, "0", 2]
    when "e" then [mday, " ", 2]
    when "j" then [yday, "0", 3]
    when "H" then [hour, "0", 2]
    when "k" then [hour, " ", 2]
    when "I" then [__hour_of_twelve__, "0", 2]
    when "l" then [__hour_of_twelve__, " ", 2]
    when "M" then [min, "0", 2]
    when "S" then [sec, "0", 2]
    when "s" then [to_i, "0", 1]
    when "u" then [wday == 0 ? 7 : wday, "0", 1]
    when "w" then [wday, "0", 1]
    when "U" then [(yday + 6 - wday) / 7, "0", 2]
    when "W" then [(yday + 6 - (wday + 6) % 7) / 7, "0", 2]
    when "V" then [__week_of_year__[1], "0", 2]
    when "G" then [__week_of_year__[0], "0", 4]
    when "g" then [__week_of_year__[0] % 100, "0", 2]
    else [nil, "0", 1]
    end
  end
  private :__directive_number__

  def __hour_of_twelve__
    held = hour % 12
    held == 0 ? 12 : held
  end
  private :__hour_of_twelve__

  # The year and week the ISO calendar counts this day in, where a week runs
  # Monday to Sunday and belongs to the year holding its Thursday.
  def __week_of_year__
    weekday = wday == 0 ? 7 : wday
    thursday = yday - weekday + 4
    counted = year
    if thursday < 1
      counted -= 1
      thursday += Time.days_in_year(counted)
    elsif thursday > Time.days_in_year(year)
      thursday -= Time.days_in_year(year)
      counted += 1
    end
    [counted, (thursday - 1) / 7 + 1]
  end
  private :__week_of_year__

  # A number written with the padding and width a directive asked for.
  def __padded_number__(counted, flags, width, pad, places)
    pad = __padding_of__(flags, pad)
    places = width if width > places
    negative = counted < 0
    digits = counted.abs.to_s
    return negative ? "-#{digits}" : digits if flags.include?("-")
    room = places - (negative ? 1 : 0)
    digits = digits.rjust(room, pad) if digits.length < room
    negative ? "-#{digits}" : digits
  end
  private :__padded_number__

  # The fraction of a second, written to the given number of digits, rounded
  # rather than cut down.
  def fraction_digits(places)
    scaled = (@fraction * 10 ** places).round.to_i
    scaled.to_s.rjust(places, "0")
  end
  private :fraction_digits

  # The fraction of a second, written to the number of places asked for.
  def __fraction_text__(places)
    scaled = (@fraction * 10 ** places).to_i
    scaled.to_s.rjust(places, "0")
  end
  private :__fraction_text__

  # The offset from UTC, written the way `%z` and its colon forms ask for. A
  # time in UTC written with the `-` flag reads as the unknown local offset
  # RFC 3339 spells `-0000`.
  def __offset_text__(flags, width, colons)
    counted = utc_offset.round
    sign = counted < 0 ? "-" : "+"
    sign = "-" if flags.include?("-") && @utc
    counted = counted.abs
    hours = counted / 3600
    minutes = counted % 3600 / 60
    seconds = counted % 60
    tail = case colons
           when 0 then "%02d" % minutes
           when 1 then ":%02d" % minutes
           when 2 then ":%02d:%02d" % [minutes, seconds]
           else
             if seconds > 0
               ":%02d:%02d" % [minutes, seconds]
             elsif minutes > 0
               ":%02d" % minutes
             else
               ""
             end
           end
    if flags.include?("_")
      return "#{sign}#{hours}#{tail}".rjust(width, " ")
    end
    room = [2, width - 1 - tail.length].max
    "#{sign}#{hours.to_s.rjust(room, "0")}#{tail}"
  end
  private :__offset_text__

  # How many days a year holds, which is one more in a leap year.
  def self.days_in_year(counted)
    leap = counted % 4 == 0 && (counted % 100 != 0 || counted % 400 == 0)
    leap ? 366 : 365
  end

  # The offset from UTC as `+0900`, or as `+09:00` when asked for the spelling
  # that carries a colon.
  def offset_label(with_colon)
    counted = utc_offset
    sign = counted < 0 ? "-" : "+"
    counted = counted.abs
    hours = (counted / 3600).to_s.rjust(2, "0")
    minutes = (counted % 3600 / 60).to_s.rjust(2, "0")
    "#{sign}#{hours}#{with_colon ? ":" : ""}#{minutes}"
  end
  private :offset_label

  def asctime
    strftime("%a %b %e %H:%M:%S %Y")
  end

  def ctime
    asctime
  end

  # A time writes itself with nothing but ASCII in it, which is the encoding
  # Ruby tags the result with.
  def to_s
    "#{strftime("%Y-%m-%d %H:%M:%S")} #{zone_label}".force_encoding(Encoding::US_ASCII)
  end

  def inspect
    "#{strftime("%Y-%m-%d %H:%M:%S")}#{fraction_label} #{zone_label}".force_encoding(Encoding::US_ASCII)
  end

  # What follows the seconds in a rendering: `UTC` for a time read in UTC,
  # and the offset otherwise.
  def zone_label
    @utc && @offset.nil? ? "UTC" : offset_label(false)
  end
  private :zone_label

  # The fraction of a second a rendering shows, with the trailing zeros left
  # off and nothing at all for a whole second.
  def fraction_label
    return "" if @fraction == 0
    digits = fraction_digits(9).sub(/0+\z/, "")
    ".#{digits}"
  end
  private :fraction_label

  def iso8601(fraction = 0)
    xmlschema(fraction)
  end

  def xmlschema(fraction = 0)
    written = "#{year_label}-#{padded(mon)}-#{padded(day)}"
    written += "T#{padded(hour)}:#{padded(min)}:#{padded(sec)}"
    written += ".#{fraction_digits(fraction)}" if fraction > 0
    "#{written}#{@utc && @offset.nil? ? "Z" : offset_label(true)}"
  end

  # The year an ISO-8601 rendering shows, which runs to four digits at least
  # and carries a sign when it falls before the common era.
  def year_label
    counted = year
    return "-#{counted.abs.to_s.rjust(4, "0")}" if counted < 0
    counted.to_s.rjust(4, "0")
  end
  private :year_label

  def padded(field)
    field.to_s.rjust(2, "0")
  end
  private :padded

end

"##;
