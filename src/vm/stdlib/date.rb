# A calendar date, held as the Julian Day Number it stands for together with
# the day the Gregorian calendar takes over from the Julian one.
class Date
  include Comparable

  class Error < ArgumentError
  end

  # A value that compares greater or less than every date, which is what the
  # calendars that never reform are written as.
  class Infinity < Numeric
    include Comparable

    def initialize(direction = 1)
      @direction = direction <=> 0
    end

    def direction
      @direction
    end

    def zero?
      false
    end

    def finite?
      false
    end

    def infinite?
      @direction == 0 ? nil : @direction
    end

    def nan?
      @direction == 0
    end

    def abs
      Infinity.new
    end

    def -@
      Infinity.new(-@direction)
    end

    def +@
      Infinity.new(@direction)
    end

    def <=>(other)
      return @direction <=> other.direction if other.is_a?(Infinity)
      return @direction if other.is_a?(Numeric)
      nil
    end

    def coerce(other)
      return [other.direction, @direction] if other.is_a?(Infinity)
      [-@direction, @direction]
    end

    def to_f
      @direction * Float::INFINITY
    end
  end

  ITALY = 2299161
  ENGLAND = 2361222
  JULIAN = Infinity.new
  GREGORIAN = Infinity.new(-1)
  MONTHNAMES = [nil, "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November",
                "December"].each { |name| name.freeze }.freeze
  DAYNAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
              "Saturday"].each { |name| name.freeze }.freeze
  ABBR_MONTHNAMES = [nil, "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
                     "Sep", "Oct", "Nov", "Dec"].each { |name| name.freeze }.freeze
  ABBR_DAYNAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri",
                   "Sat"].each { |name| name.freeze }.freeze
  VERSION = "3.4.1"

  # ── Building ─────────────────────────────────────────────────────────────

  def self.from_jd(jd, start)
    made = allocate
    made.instance_variable_set(:@jd, jd)
    made.instance_variable_set(:@start, start)
    made
  end

  def self.jd(number = 0, start = ITALY)
    from_jd(number.to_i, start)
  end

  def self.civil(year = -4712, month = 1, day = 1, start = ITALY)
    found = valid_civil_jd(year, month, day, start)
    raise Date::Error, "invalid date" if found.nil?
    from_jd(found, start)
  end

  def self.new(year = -4712, month = 1, day = 1, start = ITALY)
    civil(year, month, day, start)
  end

  def self.ordinal(year = -4712, day = 1, start = ITALY)
    found = valid_ordinal_jd(year, day, start)
    raise Date::Error, "invalid date" if found.nil?
    from_jd(found, start)
  end

  def self.commercial(year = -4712, week = 1, weekday = 1, start = ITALY)
    raise Date::Error, "invalid date" if week < 1 || week > 53
    raise Date::Error, "invalid date" if weekday < 1 || weekday > 7
    fourth = civil(year, 1, 4, start)
    monday = fourth.jd - (fourth.cwday - 1)
    found = monday + (week - 1) * 7 + (weekday - 1)
    made = from_jd(found, start)
    raise Date::Error, "invalid date" unless made.cwyear == year
    made
  end

  def self._iso8601(text)
    return {} if text.nil?
    unless text.is_a?(String)
      raise TypeError, "no implicit conversion of #{text.class} into String"
    end
    text = text.to_s
    if text =~ /\A(-?\d{4,})-(\d{2})-(\d{2})([Tt ].*)?\z/
      return { year: $1.to_i, mon: $2.to_i, mday: $3.to_i }
    end
    if text =~ /\A(-?\d{4,})(\d{2})(\d{2})([Tt ].*)?\z/
      return { year: $1.to_i, mon: $2.to_i, mday: $3.to_i }
    end
    {}
  end

  def self._rfc3339(text)
    return {} if text.nil?
    unless text.is_a?(String)
      raise TypeError, "no implicit conversion of #{text.class} into String"
    end
    text = text.to_s
    return {} unless text =~ /\A(-?\d{4,})-(\d{2})-(\d{2})[Tt ]/
    { year: $1.to_i, mon: $2.to_i, mday: $3.to_i }
  end

  def self.iso8601(text = "-4712-01-01", start = ITALY)
    fields = _iso8601(text)
    raise Date::Error, "invalid date" if fields.empty?
    Date.civil(fields[:year], fields[:mon], fields[:mday], start)
  end

  def self.rfc3339(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    fields = _rfc3339(text)
    raise Date::Error, "invalid date" if fields.empty?
    Date.civil(fields[:year], fields[:mon], fields[:mday], start)
  end

  # A date written as text, read back into the fields it names. Returns an
  # empty hash when the text names no date at all.
  def self._parse(text, comp = true)
    text = coerce_to_string(text)
    clock = {}
    text = take_clock(text, clock)
    tokens = tokenize(text)
    found = {}
    numbers = []
    month = nil
    weekday = nil
    tokens.each do |kind, value|
      if kind == :number
        numbers.push(value)
        next
      end
      if ["st", "nd", "rd", "th"].include?(value) && !numbers.empty?
        found[:mday] = numbers.pop.to_i
        next
      end
      picked = month_index(value)
      if picked.nil?
        named = dayname_index(value)
        weekday = named unless named.nil?
      else
        month = picked
      end
    end
    found[:mon] = month unless month.nil?
    found[:wday] = weekday unless weekday.nil?
    if month.nil? && weekday.nil? && !found.key?(:mday)
      bare = read_bare_numbers(numbers, comp)
      return bare.empty? && clock.empty? ? {} : bare.merge(clock)
    end
    numbers.each do |number|
      if number.start_with?("-") || number.length >= 4
        found[:year] = number.to_i
      elsif found.key?(:mday)
        found[:year] = complete_year(number.to_i, comp)
      else
        found[:mday] = number.to_i
      end
    end
    found.merge(clock)
  end

  # The time of day and zone a date string carries, taken out of the text so
  # what remains is the date alone.
  def self.take_clock(text, clock)
    # A zone follows the clock either as a signed offset or as one of the
    # names RFC 822 lists.
    # RFC 2822 lets folding whitespace sit either side of the colons, so
    # "09 :   55  :  06" names the same clock as "09:55:06".
    pattern = /(\d{1,2})\s*:\s*(\d{2})(\s*:\s*(\d{2}(\.\d+)?))?\s*(z|Z|[+-]\d{2}:?\d{2}|[A-Za-z]{2,3})?/
    return text if (text =~ pattern).nil?
    clock[:hour] = $1.to_i
    clock[:min] = $2.to_i
    unless $4.nil?
      # The digits after the point name an exact fraction, so ".52" is 13/25
      # rather than the nearest Float to it.
      if $4.include?(".")
        whole, fraction = $4.split(".")
        clock[:sec] = whole.to_i + Rational(fraction.to_i, 10 ** fraction.length)
      else
        clock[:sec] = $4.to_i
      end
    end
    unless $6.nil?
      clock[:offset] = (DateTime.read_offset(zone_text($6)) * 86400).to_i
    end
    text.sub(pattern, " ")
  end

  # The offsets RFC 822 gives the North American zone names, in hours from
  # UTC. The military single letters are not listed: RFC 822 got their signs
  # wrong, and RFC 2822 tells a reader to treat them as UTC.
  NAMED_ZONE_HOURS = {
    "ut" => 0, "gmt" => 0, "utc" => 0,
    "est" => -5, "edt" => -4,
    "cst" => -6, "cdt" => -5,
    "mst" => -7, "mdt" => -6,
    "pst" => -8, "pdt" => -7
  }

  # A zone written beside a time, spelled the way `read_offset` wants it.
  def self.zone_text(written)
    lowered = written.downcase
    return "+0000" if lowered == "z"
    hours = NAMED_ZONE_HOURS[lowered]
    return written if hours.nil?
    sign = hours < 0 ? "-" : "+"
    sign + hours.abs.to_s.rjust(2, "0") + "00"
  end

  # The words and numbers a date string is built from, paired with which of
  # the two each one is.
  def self.tokenize(text)
    lowered = text.downcase
    tokens = []
    index = 0
    while index < lowered.length
      character = lowered[index]
      if letter?(character)
        finish = index
        finish += 1 while finish < lowered.length && letter?(lowered[finish])
        tokens.push([:word, lowered[index...finish]])
        index = finish
      elsif digit?(character) || signs_a_number(lowered, index)
        finish = index + 1
        finish += 1 while finish < lowered.length && digit?(lowered[finish])
        tokens.push([:number, lowered[index...finish]])
        index = finish
      else
        index += 1
      end
    end
    tokens
  end

  # A `-` opens a negative year only where no digit precedes it, so the
  # hyphens of `10-01-2007` stay separators.
  def self.signs_a_number(lowered, index)
    return false unless lowered[index] == "-"
    return false if index + 1 >= lowered.length || !digit?(lowered[index + 1])
    index == 0 || !digit?(lowered[index - 1])
  end

  def self.letter?(character)
    character >= "a" && character <= "z"
  end

  def self.digit?(character)
    character >= "0" && character <= "9"
  end

  def self.month_index(word)
    (1..12).find { |number| word.start_with?(ABBR_MONTHNAMES[number].downcase) }
  end

  def self.dayname_index(word)
    (0..6).find { |number| word.start_with?(ABBR_DAYNAMES[number].downcase) }
  end

  # Digits with no month or day name beside them, read the way Ruby reads a
  # bare numeric date.
  def self.read_bare_numbers(numbers, comp)
    return {} if numbers.empty?
    if numbers.length >= 3
      year, month, day = numbers[0], numbers[1], numbers[2]
      year, month, day = numbers[2], numbers[1], numbers[0] if numbers[2].length == 4
      return { year: complete_year(year.to_i, comp && year.length <= 2),
               mon: month.to_i, mday: day.to_i }
    end
    return {} if numbers.length == 2
    digits = numbers[0]
    case digits.length
    when 2 then { mday: digits.to_i }
    when 3 then { yday: digits.to_i }
    when 4 then { mon: digits[0, 2].to_i, mday: digits[2, 2].to_i }
    when 5 then { year: complete_year(digits[0, 2].to_i, comp), yday: digits[2, 3].to_i }
    when 6 then { year: complete_year(digits[0, 2].to_i, comp),
                  mon: digits[2, 2].to_i, mday: digits[4, 2].to_i }
    when 7 then { year: digits[0, 4].to_i, yday: digits[4, 3].to_i }
    when 8 then { year: digits[0, 4].to_i, mon: digits[4, 2].to_i,
                  mday: digits[6, 2].to_i }
    else {}
    end
  end

  # A year written with two digits, moved into the window 1969 through 2068
  # the way Ruby completes one.
  def self.complete_year(value, comp)
    return value unless comp
    return value if value >= 100 || value < 0
    value < 69 ? 2000 + value : 1900 + value
  end

  def self.coerce_to_string(text)
    return text.to_s if text.is_a?(String)
    if text.respond_to?(:to_str)
      converted = text.to_str
      return converted.to_s if converted.is_a?(String)
    end
    raise TypeError, "no implicit conversion of #{text.class} into String"
  end

  def self.parse(text = "-4712-01-01", comp = true, start = ITALY)
    fields = _parse(text, comp)
    raise Date::Error, "invalid date" if fields.empty?
    return Date.ordinal(fields[:year] || Date.today(start).year, fields[:yday], start) if fields[:yday]
    if fields.key?(:wday) && !fields.key?(:mon) && !fields.key?(:mday) &&
       !fields.key?(:year)
      now = Date.today(start)
      landing = fields[:wday] == 0 ? 7 : fields[:wday]
      return Date.commercial(now.cwyear, now.cweek, landing, start)
    end
    now = Date.today(start)
    year = fields[:year] || now.year
    month = fields[:mon] || (fields.key?(:year) ? 1 : now.month)
    Date.civil(year, month, fields[:mday] || 1, start)
  end

  # A date read out of text by a strftime template, returned as the fields
  # the template named, or nil when the text does not match.
  def self._strptime(text, format = "%F")
    text = coerce_to_string(text)
    format = expand_format(format.to_s)
    found = {}
    at = 0
    index = 0
    while index < format.length
      character = format[index]
      if character != "%"
        if whitespace?(character)
          at += 1 while at < text.length && whitespace?(text[at])
          index += 1
          next
        end
        return nil unless text[at] == character
        at += 1
        index += 1
        next
      end
      index += 1
      return nil if index >= format.length
      directive = format[index]
      index += 1
      while directive == "-" || directive == "_" || directive == "0" ||
            directive == "^" || digit?(directive)
        return nil if index >= format.length
        directive = format[index]
        index += 1
      end
      at = read_directive(directive, text, at, found)
      return nil if at.nil?
    end
    found
  end

  # A template with its combining directives written out in the primitive
  # ones they stand for.
  def self.expand_format(format)
    grown = true
    while grown
      grown = false
      built = ""
      index = 0
      while index < format.length
        if format[index] == "%" && index + 1 < format.length
          expanded = strptime_expansion(format[index + 1])
          if expanded.nil?
            built = built + format[index, 2]
          else
            built = built + expanded
            grown = true
          end
          index += 2
        else
          built = built + format[index]
          index += 1
        end
      end
      format = built
    end
    format
  end

  def self.strptime_expansion(directive)
    case directive
    when "c" then "%a %b %e %H:%M:%S %Y"
    when "D", "x" then "%m/%d/%y"
    when "F" then "%Y-%m-%d"
    when "v" then "%e-%b-%Y"
    when "+" then "%a %b %e %H:%M:%S %Z %Y"
    when "T", "X" then "%H:%M:%S"
    when "R" then "%H:%M"
    when "r" then "%I:%M:%S %p"
    end
  end

  # Read what one directive names out of the text, record it, and answer
  # where reading stopped. Answers nil when the text does not match.
  def self.read_directive(directive, text, at, found)
    case directive
    when "Y" then read_into(found, :year, text, at, 10, true)
    when "G" then read_into(found, :cwyear, text, at, 10, true)
    when "s" then read_into(found, :epoch, text, at, 12, true)
    when "C" then read_into(found, :century, text, at, 2, false)
    when "y" then read_into(found, :year_short, text, at, 2, false)
    when "g" then read_into(found, :cwyear_short, text, at, 2, false)
    when "m" then read_into(found, :mon, text, at, 2, false)
    when "d", "e" then read_into(found, :mday, text, at, 2, false)
    when "j" then read_into(found, :yday, text, at, 3, false)
    when "U" then read_into(found, :week_sunday, text, at, 2, false)
    when "W" then read_into(found, :week_monday, text, at, 2, false)
    when "V" then read_into(found, :cweek, text, at, 2, false)
    when "u" then read_into(found, :cwday, text, at, 1, false)
    when "w" then read_into(found, :wday, text, at, 1, false)
    when "H", "k", "I", "l" then read_into(found, :hour, text, at, 2, false)
    when "M" then read_into(found, :min, text, at, 2, false)
    when "S" then read_into(found, :sec, text, at, 2, false)
    when "L", "N", "Q" then read_into(found, :ignored, text, at, 12, true)
    when "A", "a" then read_name(found, :wday, ABBR_DAYNAMES, text, at)
    when "B", "b", "h" then read_name(found, :mon, ABBR_MONTHNAMES, text, at)
    when "p", "P" then read_word(text, at)
    when "Z", "z" then read_zone(found, text, at)
    when "n", "t" then skip_whitespace(text, at)
    when "%" then text[at] == "%" ? at + 1 : nil
    end
  end

  def self.read_into(found, field, text, at, limit, signed)
    at = skip_whitespace(text, at)
    start = at
    at += 1 if signed && (text[at] == "-" || text[at] == "+")
    digits = 0
    while at < text.length && digit?(text[at]) && digits < limit
      at += 1
      digits += 1
    end
    return nil if digits == 0
    found[field] = text[start...at].to_i
    at
  end

  def self.read_name(found, field, names, text, at)
    at = skip_whitespace(text, at)
    finish = at
    finish += 1 while finish < text.length && letter?(text[finish].downcase)
    return nil if finish == at
    word = text[at...finish].downcase
    picked = (0...names.length).find do |number|
      !names[number].nil? && word.start_with?(names[number].downcase)
    end
    return nil if picked.nil?
    found[field] = picked
    finish
  end

  def self.read_word(text, at)
    at = skip_whitespace(text, at)
    at += 1 while at < text.length && letter?(text[at].downcase)
    at
  end

  def self.read_zone(found, text, at)
    at = skip_whitespace(text, at)
    start = at
    at += 1 while at < text.length && !whitespace?(text[at])
    written = text[start...at]
    return at if written.empty?
    begin
      found[:offset] = (DateTime.read_offset(zone_text(written)) * 86400).to_i
    rescue Date::Error
      found[:offset] = 0
    end
    at
  end

  def self.skip_whitespace(text, at)
    at += 1 while at < text.length && whitespace?(text[at])
    at
  end

  def self.whitespace?(character)
    character == " " || character == "\t" || character == "\n"
  end

  def self.strptime(text = "-4712-01-01", format = "%F", start = ITALY)
    fields = _strptime(text, format)
    raise Date::Error, "invalid date" if fields.nil?
    assemble_strptime(fields, start)
  end

  # The date a set of strptime fields names, filling what the template left
  # out from today where Ruby does.
  def self.assemble_strptime(fields, start)
    year = fields[:year]
    if year.nil? && fields.key?(:year_short)
      year = if fields.key?(:century)
               fields[:century] * 100 + fields[:year_short]
             else
               complete_year(fields[:year_short], true)
             end
    end
    if fields.key?(:mon) || fields.key?(:mday)
      now = Date.today(start)
      opening = year.nil? ? now.month : 1
      return Date.civil(year || now.year, fields[:mon] || opening, fields[:mday] || 1, start)
    end
    return Date.ordinal(year || Date.today(start).year, fields[:yday], start) if fields.key?(:yday)
    cwyear = fields[:cwyear]
    if cwyear.nil? && fields.key?(:cwyear_short)
      cwyear = complete_year(fields[:cwyear_short], true)
    end
    if !cwyear.nil? || fields.key?(:cweek) || fields.key?(:cwday)
      now = Date.today(start)
      return Date.commercial(cwyear || now.cwyear, fields[:cweek] || 1,
                             fields[:cwday] || 1, start)
    end
    if fields.key?(:week_sunday) || fields.key?(:week_monday) ||
       (!year.nil? && fields.key?(:wday))
      return week_of(year || Date.today(start).year, fields, start)
    end
    if fields.key?(:wday)
      now = Date.today(start)
      return Date.from_jd(now.jd - now.wday + fields[:wday], start)
    end
    return Date.civil(year, 1, 1, start) unless year.nil?
    Date.civil(-4712, 1, 1, start)
  end

  # A date named by a week number counted from the first week of a year,
  # with weeks opening on Sunday for %U and on Monday for %W.
  def self.week_of(year, fields, start)
    opening = Date.civil(year, 1, 1, start)
    if fields.key?(:week_monday)
      base = opening.jd - ((opening.wday + 6) % 7)
      offset = fields.key?(:wday) ? (fields[:wday] + 6) % 7 : 0
      week = fields[:week_monday]
    else
      base = opening.jd - opening.wday
      offset = fields[:wday] || 0
      week = fields[:week_sunday] || 0
    end
    Date.from_jd(base + week * 7 + offset, start)
  end

  def self.today(start = ITALY)
    now = Time.now
    civil(now.year, now.month, now.day, start)
  end

  def self.valid_civil?(year, month, day, start = ITALY)
    !valid_civil_jd(year, month, day, start).nil?
  end

  def self.valid_date?(year, month, day, start = ITALY)
    valid_civil?(year, month, day, start)
  end

  def self.valid_jd?(number, start = ITALY)
    number.is_a?(Numeric)
  end

  def self.valid_ordinal?(year, day, start = ITALY)
    !valid_ordinal_jd(year, day, start).nil?
  end

  def self.valid_ordinal_jd(year, day, start)
    return nil unless day.is_a?(Integer)
    first = month_first_jd(year, 1, start)
    last = month_first_jd(year + 1, 1, start) - 1
    found = day < 0 ? last + day + 1 : first + day - 1
    return nil if found < first || found > last
    found
  end

  # How many commercial weeks a year holds, which is 53 when it opens on a
  # Thursday and 52 otherwise, with a leap year that opens on a Wednesday
  # reaching 53 too.
  def self.commercial_weeks(year)
    opening = (gregorian_to_jd(year, 1, 1) + 1) % 7
    return 53 if opening == 4
    return 53 if opening == 3 && gregorian_leap?(year)
    52
  end

  def self.valid_commercial?(year, week, weekday, start = ITALY)
    return false unless week.is_a?(Integer) && weekday.is_a?(Integer)
    weekday = 8 + weekday if weekday < 0
    return false if weekday < 1 || weekday > 7
    weeks = commercial_weeks(year)
    week = weeks + week + 1 if week < 0
    week >= 1 && week <= weeks
  end

  def self.leap?(year)
    gregorian_leap?(year)
  end

  def self.gregorian_leap?(year)
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
  end

  def self.julian_leap?(year)
    year % 4 == 0
  end

  # The Julian Day Number a set of calendar fields stands for, or nil when
  # those fields name no day at all.
  def self.valid_civil_jd(year, month, day, start)
    return nil unless month.is_a?(Integer) && day.is_a?(Integer)
    if month < 0
      month = 13 + month
    end
    return nil if month < 1 || month > 12
    if day < 0
      last = month_last_jd(year, month, start)
      found = last + day + 1
      return nil if found < month_first_jd(year, month, start)
      return found
    end
    counted = day
    return nil if counted < 1 || counted > days_in_month(year, month, start)
    found = gregorian_to_jd(year, month, counted)
    return found if gregorian_from?(found, start)
    found = julian_to_jd(year, month, counted)
    return found unless gregorian_from?(found, start)
    nil
  end

  # The Julian Day Number the first day of a month falls on, read on
  # whichever calendar governs it.
  def self.month_first_jd(year, month, start)
    found = gregorian_to_jd(year, month, 1)
    return found if gregorian_from?(found, start)
    julian_to_jd(year, month, 1)
  end

  def self.month_last_jd(year, month, start)
    landing_year = month == 12 ? year + 1 : year
    landing_month = month == 12 ? 1 : month + 1
    month_first_jd(landing_year, landing_month, start) - 1
  end

  def self.days_in_month(year, month, start = ITALY)
    return 31 if [1, 3, 5, 7, 8, 10, 12].include?(month)
    return 30 if [4, 6, 9, 11].include?(month)
    reformed = gregorian_from?(gregorian_to_jd(year, 3, 1), start)
    leaping = reformed ? gregorian_leap?(year) : julian_leap?(year)
    leaping ? 29 : 28
  end

  # Whether a Julian Day Number falls on the Gregorian side of a calendar
  # reform. The endless calendars stand in for a reform that never happened
  # and one that always had, so they answer without comparing.
  def self.gregorian_from?(jd, start)
    return start.direction < 0 if start.is_a?(Infinity)
    jd >= start
  end

  def self.gregorian_to_jd(year, month, day)
    shift = (14 - month) / 12
    counted = year + 4800 - shift
    moved = month + 12 * shift - 3
    day + (153 * moved + 2) / 5 + 365 * counted + counted / 4 - counted / 100 +
      counted / 400 - 32045
  end

  def self.julian_to_jd(year, month, day)
    shift = (14 - month) / 12
    counted = year + 4800 - shift
    moved = month + 12 * shift - 3
    day + (153 * moved + 2) / 5 + 365 * counted + counted / 4 - 32083
  end

  def self.jd_to_gregorian(jd)
    a = jd + 32044
    b = (4 * a + 3) / 146097
    c = a - 146097 * b / 4
    d = (4 * c + 3) / 1461
    e = c - 1461 * d / 4
    m = (5 * e + 2) / 153
    day = e - (153 * m + 2) / 5 + 1
    month = m + 3 - 12 * (m / 10)
    year = 100 * b + d - 4800 + m / 10
    [year, month, day]
  end

  def self.jd_to_julian(jd)
    c = jd + 32082
    d = (4 * c + 3) / 1461
    e = c - 1461 * d / 4
    m = (5 * e + 2) / 153
    day = e - (153 * m + 2) / 5 + 1
    month = m + 3 - 12 * (m / 10)
    year = d - 4800 + m / 10
    [year, month, day]
  end

  # ── What a date stands for ───────────────────────────────────────────────

  def jd
    @jd
  end

  def start
    @start
  end

  def julian?
    !Date.gregorian_from?(@jd, @start)
  end

  def gregorian?
    !self.julian?
  end

  def fields
    return @fields unless @fields.nil?
    @fields = self.julian? ? Date.jd_to_julian(@jd) : Date.jd_to_gregorian(@jd)
  end
  private :fields

  def year
    self.fields[0]
  end

  def month
    self.fields[1]
  end

  def mon
    self.fields[1]
  end

  def day
    self.fields[2]
  end

  def mday
    self.fields[2]
  end

  def wday
    (@jd + 1) % 7
  end

  def yday
    first = self.julian? ? Date.julian_to_jd(self.year, 1, 1) : Date.gregorian_to_jd(self.year, 1, 1)
    @jd - first + 1
  end

  def cwday
    counted = self.wday
    counted == 0 ? 7 : counted
  end

  def cweek
    (@jd - self.commercial_monday) / 7 + 1
  end

  def cwyear
    thursday = @jd + (4 - self.cwday)
    held = Date.from_jd(thursday, @start)
    held.year
  end

  # The Monday that opens the commercial year this date falls in.
  def commercial_monday
    thursday = @jd + (4 - self.cwday)
    held = Date.from_jd(thursday, @start)
    fourth = Date.civil(held.year, 1, 4, @start)
    fourth.jd - (fourth.cwday - 1)
  end
  private :commercial_monday

  def mjd
    @jd - 2400001
  end

  def ajd
    Rational(2 * @jd - 1, 2)
  end

  def amjd
    Rational(2 * self.mjd, 2)
  end

  def day_fraction
    Rational(0, 1)
  end

  # The clock a date carries. A plain Date stands for midnight with no offset
  # from UTC, and DateTime answers the time of day and offset it holds.
  def clock_fraction
    Rational(0, 1)
  end
  private :clock_fraction

  def zone_offset
    Rational(0, 1)
  end
  private :zone_offset

  def clock_seconds
    self.clock_fraction * 86400
  end
  private :clock_seconds

  def clock_hour
    self.clock_seconds.floor / 3600
  end
  private :clock_hour

  def clock_minute
    self.clock_seconds.floor / 60 % 60
  end
  private :clock_minute

  def clock_second
    self.clock_seconds.floor % 60
  end
  private :clock_second

  def clock_sub_second
    self.clock_seconds - self.clock_seconds.floor
  end
  private :clock_sub_second

  # The part of a second below one, written to as many digits as asked for.
  def sub_second_digits(count)
    (self.clock_sub_second * 10 ** count).floor.to_s.rjust(count, "0")
  end
  private :sub_second_digits

  # The offset from UTC, written with as many separators as the directive
  # asking for it wants.
  def offset_text(separators)
    total = (self.zone_offset * 86400).to_i
    sign = total < 0 ? "-" : "+"
    total = total.abs
    written = "#{sign}#{self.two(total / 3600)}"
    return written + self.two(total / 60 % 60) if separators == 0
    written = written + ":" + self.two(total / 60 % 60)
    return written if separators == 1
    written + ":" + self.two(total % 60)
  end
  private :offset_text

  def epoch_seconds
    ((@jd - 2440588) * 86400 + self.clock_seconds - self.zone_offset * 86400).floor
  end
  private :epoch_seconds

  def epoch_milliseconds
    (((@jd - 2440588) * 86400 + self.clock_seconds -
      self.zone_offset * 86400) * 1000).floor
  end
  private :epoch_milliseconds

  def ld
    @jd - 2299160
  end

  def leap?
    self.julian? ? Date.julian_leap?(self.year) : Date.gregorian_leap?(self.year)
  end

  def sunday?
    self.wday == 0
  end

  def monday?
    self.wday == 1
  end

  def tuesday?
    self.wday == 2
  end

  def wednesday?
    self.wday == 3
  end

  def thursday?
    self.wday == 4
  end

  def friday?
    self.wday == 5
  end

  def saturday?
    self.wday == 6
  end

  # ── Moving about ─────────────────────────────────────────────────────────

  # The same point on the calendar, moved to another day. DateTime overrides
  # this so a shifted one keeps the time of day and offset it holds.
  def rebuild(jd)
    Date.from_jd(jd, @start)
  end
  private :rebuild

  def +(count)
    raise TypeError, "expected numeric" unless count.is_a?(Numeric)
    self.rebuild(@jd + count.to_i)
  end

  def -(other)
    return @jd - other.jd if other.is_a?(Date)
    raise TypeError, "expected numeric" unless other.is_a?(Numeric)
    self.rebuild(@jd - other.to_i)
  end

  def >>(months)
    raise TypeError, "expected numeric" unless months.is_a?(Integer)
    total = self.year * 12 + (self.month - 1) + months
    landing_year = total / 12
    landing_month = total % 12 + 1
    landing_day = self.day
    limit = Date.days_in_month(landing_year, landing_month, @start)
    landing_day = limit if landing_day > limit
    found = Date.valid_civil_jd(landing_year, landing_month, landing_day, @start)
    while found.nil? && landing_day > 1
      landing_day -= 1
      found = Date.valid_civil_jd(landing_year, landing_month, landing_day, @start)
    end
    raise Date::Error, "invalid date" if found.nil?
    self.rebuild(found)
  end

  def <<(months)
    self >> -months
  end

  def next_day(count = 1)
    self + count
  end

  def prev_day(count = 1)
    self - count
  end

  def next_month(count = 1)
    self >> count
  end

  def prev_month(count = 1)
    self << count
  end

  def next_year(count = 1)
    self >> (count * 12)
  end

  def prev_year(count = 1)
    self << (count * 12)
  end

  def succ
    self + 1
  end

  def next
    self + 1
  end

  def upto(limit, &block)
    return to_enum(:upto, limit) if block.nil?
    walked = self
    while walked <= limit
      block.call(walked)
      walked = walked + 1
    end
    self
  end

  def downto(limit, &block)
    return to_enum(:downto, limit) if block.nil?
    walked = self
    while walked >= limit
      block.call(walked)
      walked = walked - 1
    end
    self
  end

  def step(limit, by = 1, &block)
    return to_enum(:step, limit, by) if block.nil?
    raise ArgumentError, "step can't be 0" if by == 0
    walked = self
    if by > 0
      while walked <= limit
        block.call(walked)
        walked = walked + by
      end
    else
      while walked >= limit
        block.call(walked)
        walked = walked + by
      end
    end
    self
  end

  # ── Which calendar a date is read on ─────────────────────────────────────

  def new_start(start = ITALY)
    self.class.from_jd(@jd, start)
  end

  def italy
    new_start(ITALY)
  end

  def england
    new_start(ENGLAND)
  end

  def julian
    new_start(JULIAN)
  end

  def gregorian
    new_start(GREGORIAN)
  end

  # ── Comparing and rendering ──────────────────────────────────────────────

  def <=>(other)
    return @jd <=> other.jd if other.is_a?(Date)
    return @jd <=> other if other.is_a?(Numeric)
    nil
  end

  def ==(other)
    other.is_a?(Date) && @jd == other.jd
  end

  def ===(other)
    return @jd == other.jd if other.is_a?(Date)
    return @jd == other if other.is_a?(Numeric)
    false
  end

  def eql?(other)
    other.is_a?(Date) && @jd == other.jd
  end

  def hash
    @jd.hash
  end

  def to_s
    "#{self.padded_year}-#{self.two(self.month)}-#{self.two(self.day)}"
  end

  def inspect
    "#<Date: #{to_s} ((#{@jd}j),(0/1),(#{@start}j))>"
  end

  def iso8601
    to_s
  end

  def xmlschema
    to_s
  end

  def rfc3339
    "#{to_s}T00:00:00+00:00"
  end

  def asctime
    "#{ABBR_DAYNAMES[self.wday]} #{ABBR_MONTHNAMES[self.month]} #{self.day.to_s.rjust(2)} 00:00:00 #{self.padded_year}"
  end

  def ctime
    asctime
  end

  def to_date
    self
  end

  def to_datetime
    DateTime.from_parts(@jd, Rational(0, 1), Rational(0, 1), @start)
  end

  def to_time
    Time.local(self.year, self.month, self.day)
  end

  def deconstruct_keys(keys)
    all = { year: self.year, month: self.month, day: self.day, yday: self.yday, wday: self.wday }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end

  def strftime(template = "%F")
    template = template.to_s
    built = ""
    index = 0
    while index < template.length
      character = template[index]
      unless character == "%"
        built = built + character
        index = index + 1
        next
      end
      index = index + 1
      flags = ""
      while index < template.length && "-_0^#".include?(template[index])
        flags = flags + template[index]
        index = index + 1
      end
      width = ""
      while index < template.length && template[index] =~ /\A[0-9]\z/
        width = width + template[index]
        index = index + 1
      end
      if index >= template.length
        built = built + "%" + flags + width
        break
      end
      directive = template[index]
      if directive == ":"
        ahead = index
        ahead += 1 while ahead < template.length && template[ahead] == ":"
        if ahead < template.length && template[ahead] == "z"
          directive = ahead - index == 1 ? ":z" : "::z"
          index = ahead
        end
      end
      index = index + 1
      built = built + self.format_directive(directive, flags, width)
    end
    built
  end

  # One `%` directive of a strftime template, with the GNU flags that pad it,
  # drop its zeros, or shift its case applied.
  def format_directive(directive, flags, width)
    return self.sub_second_digits(width.empty? ? 9 : width.to_i) if directive == "N"
    body = self.directive_body(directive)
    return body if body.nil? == false && self.directive_is_literal(directive)
    return "%" + flags + width + directive if body.nil?
    body = body.upcase if flags.include?("^")
    if flags.include?("-")
      body = body.sub(/\A[0 ]+(?=.)/, "")
      return body
    end
    filler = self.padding_character(directive, flags)
    wanted = width.empty? ? self.directive_width(directive) : width.to_i
    return body if body.length >= wanted
    if filler == "0" && (body.start_with?("-") || body.start_with?("+"))
      return body[0] + body[1..-1].rjust(wanted - 1, "0")
    end
    body.rjust(wanted, filler)
  end
  private :format_directive

  # Whether a directive writes text of its own that no flag or width touches.
  def directive_is_literal(directive)
    ["n", "t", "%", "L"].include?(directive)
  end
  private :directive_is_literal

  # The last of the `0` and `_` flags decides what pads a directive, so
  # `%0_10h` pads with spaces and `%_010h` pads with zeros.
  def padding_character(directive, flags)
    picked = nil
    flags.each_char do |flag|
      picked = "0" if flag == "0"
      picked = " " if flag == "_"
    end
    return picked unless picked.nil?
    ["e", "k", "l"].include?(directive) ? " " : self.default_filler(directive)
  end
  private :padding_character

  def default_filler(directive)
    ["A", "a", "B", "b", "h", "P", "p", "Z", "c", "v", "+", "z", ":z",
     "::z"].include?(directive) ? " " : "0"
  end
  private :default_filler

  def directive_width(directive)
    return 3 if directive == "j"
    return 4 if ["Y", "G"].include?(directive)
    return 1 if ["u", "w", "n", "t", "%", "L", "N", "s", "Q"].include?(directive)
    return 0 if ["A", "a", "B", "b", "h", "P", "p", "Z", "c", "D", "F", "R", "r",
                 "T", "X", "x", "v", "+", "z", ":z", "::z"].include?(directive)
    2
  end
  private :directive_width

  def directive_body(directive)
    case directive
    when "A" then DAYNAMES[self.wday]
    when "a" then ABBR_DAYNAMES[self.wday]
    when "B" then MONTHNAMES[self.month]
    when "b", "h" then ABBR_MONTHNAMES[self.month]
    when "C" then (self.year / 100).to_s
    when "d" then self.day.to_s
    when "e" then self.day.to_s
    when "F" then self.strftime("%Y-%m-%d")
    when "G" then self.cwyear.to_s
    when "g" then (self.cwyear % 100).to_s
    when "H", "k" then self.clock_hour.to_s
    when "I", "l" then self.twelve_hour.to_s
    when "j" then self.yday.to_s
    when "M" then self.clock_minute.to_s
    when "m" then self.month.to_s
    when "n" then "\n"
    when "P" then self.clock_hour < 12 ? "am" : "pm"
    when "p" then self.clock_hour < 12 ? "AM" : "PM"
    when "S" then self.clock_second.to_s
    when "s" then self.epoch_seconds.to_s
    when "L" then self.sub_second_digits(3)
    when "Q" then self.epoch_milliseconds.to_s
    when "t" then "\t"
    when "U" then self.week_of_year(0).to_s
    when "W" then self.week_of_year(1).to_s
    when "u" then self.cwday.to_s
    when "V" then self.cweek.to_s
    when "w" then self.wday.to_s
    when "Y" then self.year.to_s
    when "y" then (self.year % 100).to_s
    when "Z", ":z" then self.offset_text(1)
    when "::z" then self.offset_text(2)
    when "z" then self.offset_text(0)
    when "%" then "%"
    when "c" then self.strftime("%a %b %e %H:%M:%S %Y")
    when "D", "x" then self.strftime("%m/%d/%y")
    when "R" then self.strftime("%H:%M")
    when "r" then self.strftime("%I:%M:%S %p")
    when "T", "X" then self.strftime("%H:%M:%S")
    when "v" then self.strftime("%e-%^b-%Y")
    when "+" then self.strftime("%a %b %e %H:%M:%S %Z %Y")
    end
  end
  private :directive_body

  # The hour a twelve-hour clock shows, where midnight and noon both read 12.
  def twelve_hour
    shown = self.clock_hour % 12
    shown == 0 ? 12 : shown
  end
  private :twelve_hour

  # The week of the year this date falls in, counting from the first week that
  # opens on the named weekday.
  def week_of_year(opening)
    first = @jd - self.yday + 1
    offset = (first + 1) % 7
    shift = (opening - offset) % 7
    week_one = first + shift
    return 0 if @jd < week_one
    (@jd - week_one) / 7 + 1
  end
  private :week_of_year

  # The year written to four digits at least, with a sign when it falls
  # before the common era.
  def padded_year
    counted = self.year
    return "-#{counted.abs.to_s.rjust(4, "0")}" if counted < 0
    counted.to_s.rjust(4, "0")
  end
  private :padded_year

  def two(field)
    field.to_s.rjust(2, "0")
  end
  private :two
end

# A calendar day together with a time of day and an offset from UTC. The date
# is held the way Date holds one, and the time as an exact fraction of a day.
class DateTime < Date
  def self.from_parts(jd, fraction, offset, start)
    made = allocate
    made.instance_variable_set(:@jd, jd)
    made.instance_variable_set(:@fraction, fraction)
    made.instance_variable_set(:@offset, offset)
    made.instance_variable_set(:@start, start)
    made
  end

  def self.from_jd(jd, start)
    from_parts(jd, Rational(0, 1), Rational(0, 1), start)
  end

  # ── Building ─────────────────────────────────────────────────────────────

  def self.civil(year = -4712, month = 1, day = 1, hour = 0, minute = 0,
                 second = 0, offset = 0, start = ITALY)
    found = valid_civil_jd(year, month, day, start)
    raise Date::Error, "invalid date" if found.nil?
    with_clock(found, hour, minute, second, offset, start)
  end

  def self.new(year = -4712, month = 1, day = 1, hour = 0, minute = 0,
               second = 0, offset = 0, start = ITALY)
    civil(year, month, day, hour, minute, second, offset, start)
  end

  def self.jd(number = 0, hour = 0, minute = 0, second = 0, offset = 0,
              start = ITALY)
    with_clock(number.to_i, hour, minute, second, offset, start)
  end

  def self.ordinal(year = -4712, day = 1, hour = 0, minute = 0, second = 0,
                   offset = 0, start = ITALY)
    found = valid_ordinal_jd(year, day, start)
    raise Date::Error, "invalid date" if found.nil?
    with_clock(found, hour, minute, second, offset, start)
  end

  def self.commercial(year = -4712, week = 1, weekday = 1, hour = 0, minute = 0,
                      second = 0, offset = 0, start = ITALY)
    found = Date.commercial(year, week, weekday, start)
    with_clock(found.jd, hour, minute, second, offset, start)
  end

  def self.now(start = ITALY)
    moment = Time.now
    found = Date.civil(moment.year, moment.month, moment.day, start)
    fraction = Rational(moment.hour * 3600 + moment.min * 60 + moment.sec, 86400) +
      moment.subsec / 86400
    from_parts(found.jd, fraction, Rational(moment.utc_offset, 86400), start)
  end

  # A time of day fixed to a Julian Day Number. An hour, minute, or second
  # written as a negative counts back from the next unit up, and 24 hours
  # names midnight on the following day.
  def self.with_clock(jd, hour, minute, second, offset, start, leap_second = false)
    unless hour.is_a?(Integer) && minute.is_a?(Integer) && second.is_a?(Numeric)
      raise Date::Error, "invalid date"
    end
    hour = 24 + hour if hour < 0
    minute = 60 + minute if minute < 0
    second = 60 + second if second < 0
    raise Date::Error, "invalid date" if hour < 0 || hour > 24
    raise Date::Error, "invalid date" if minute < 0 || minute >= 60
    # A leap second is written as :60 in a timestamp being read, and it names
    # the moment the next second begins, which is where the fraction below
    # carries it. A datetime built field by field does not admit one.
    raise Date::Error, "invalid date" if second < 0
    if leap_second
      raise Date::Error, "invalid date" if second > 60
    else
      raise Date::Error, "invalid date" if second >= 60
    end
    if hour == 24
      raise Date::Error, "invalid date" unless minute == 0 && second == 0
      jd += 1
      hour = 0
    end
    fraction = Rational(hour, 24) + Rational(minute, 1440) + second.to_r / 86400
    from_parts(jd, fraction, read_offset(offset), start)
  end

  # An offset from UTC, given either as a fraction of a day or as the text a
  # zone is written with.
  def self.read_offset(offset)
    return offset.to_r if offset.is_a?(Numeric)
    raise Date::Error, "invalid date" unless offset.is_a?(String)
    text = offset.to_s
    return Rational(0, 1) if ["Z", "z", "UTC", "GMT", "UT"].include?(text)
    unless text =~ /\A([+-])(\d{2}):?(\d{2})?:?(\d{2})?\z/
      raise Date::Error, "invalid date"
    end
    seconds = $2.to_i * 3600 + $3.to_i * 60 + $4.to_i
    seconds = -seconds if $1 == "-"
    Rational(seconds, 86400)
  end

  # ── Reading text ─────────────────────────────────────────────────────────

  def self.parse(text = "-4712-01-01T00:00:00+00:00", comp = true, start = ITALY)
    fields = _parse(text, comp)
    raise Date::Error, "invalid date" if fields.empty?
    Date.parse(text, comp, start).to_datetime.with_fields(fields)
  end

  def self.strptime(text = "-4712-01-01T00:00:00+00:00", format = "%FT%T%z",
                    start = ITALY)
    fields = _strptime(text, format)
    raise Date::Error, "invalid date" if fields.nil?
    assemble_strptime(fields, start).to_datetime.with_fields(fields)
  end

  def self.iso8601(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    read_written(text, start) { |held| Date._iso8601(held) }
  end

  def self.rfc3339(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    read_written(text, start) { |held| Date._rfc3339(held) }
  end

  def self.xmlschema(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    iso8601(text, start)
  end

  def self.jisx0301(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    iso8601(text, start)
  end

  def self.httpdate(text = "Mon, 01 Jan -4712 00:00:00 GMT", start = ITALY)
    read_written(text, start) { |held| Date._parse(held) }
  end

  def self.rfc2822(text = "Mon, 1 Jan -4712 00:00:00 +0000", start = ITALY)
    read_written(text, start) { |held| Date._parse(held) }
  end

  def self.rfc822(text = "Mon, 1 Jan -4712 00:00:00 +0000", start = ITALY)
    rfc2822(text, start)
  end

  # Text read into a date by the block, with the time of day and offset the
  # same text carries laid over it.
  def self.read_written(text, start)
    raise Date::Error, "invalid date" if text.nil?
    fields = yield(text)
    raise Date::Error, "invalid date" if fields.nil? || fields.empty?
    clock = _parse(text)
    Date.civil(fields[:year], fields[:mon], fields[:mday], start)
      .to_datetime
      .with_fields(clock)
  end

  # ── What a datetime stands for ───────────────────────────────────────────

  def clock_fraction
    @fraction
  end
  private :clock_fraction

  def zone_offset
    @offset
  end
  private :zone_offset

  def hour
    self.clock_hour
  end

  def min
    self.clock_minute
  end

  def minute
    self.clock_minute
  end

  def sec
    self.clock_second
  end

  def second
    self.clock_second
  end

  def sec_fraction
    self.clock_sub_second
  end

  def second_fraction
    self.clock_sub_second
  end

  def offset
    @offset
  end

  def day_fraction
    @fraction
  end

  def zone
    self.offset_text(1)
  end

  # The same instant, read against another offset from UTC.
  def new_offset(offset = 0)
    wanted = DateTime.read_offset(offset)
    moved = @jd + @fraction - @offset + wanted
    landing = moved.floor
    DateTime.from_parts(landing, moved - landing, wanted, @start)
  end

  def new_start(start = ITALY)
    DateTime.from_parts(@jd, @fraction, @offset, start)
  end

  # The time of day and offset a parsed set of fields names, laid over this
  # datetime. Fields the text left out keep what they already hold.
  def with_fields(fields)
    offset = fields[:offset].nil? ? @offset : Rational(fields[:offset], 86400)
    DateTime.with_clock(@jd, fields[:hour] || 0, fields[:min] || 0,
                        fields[:sec] || 0, offset, @start, true)
  end

  def rebuild(jd)
    DateTime.from_parts(jd, @fraction, @offset, @start)
  end
  private :rebuild

  # ── Moving about ─────────────────────────────────────────────────────────

  def +(count)
    raise TypeError, "expected numeric" unless count.is_a?(Numeric)
    moved = @jd + @fraction + count.to_r
    landing = moved.floor
    DateTime.from_parts(landing, moved - landing, @offset, @start)
  end

  def -(other)
    return (@jd + @fraction) - (other.jd + other.day_fraction) if other.is_a?(Date)
    raise TypeError, "expected numeric" unless other.is_a?(Numeric)
    self + (-other.to_r)
  end

  # The point in time this datetime names, counted in days from the same
  # place whatever offset it is written against.
  def absolute_day
    @jd + @fraction - @offset
  end

  def <=>(other)
    return self.absolute_day <=> other.absolute_day if other.is_a?(DateTime)
    return self.absolute_day <=> (other.jd + other.day_fraction) if other.is_a?(Date)
    return self.absolute_day <=> other if other.is_a?(Numeric)
    nil
  end

  def ==(other)
    return false unless other.is_a?(Date)
    self.absolute_day == (other.is_a?(DateTime) ? other.absolute_day :
                          other.jd + other.day_fraction)
  end

  def eql?(other)
    self == other
  end

  def hash
    self.absolute_day.hash
  end

  # ── Writing text ─────────────────────────────────────────────────────────

  def strftime(template = "%FT%T%:z")
    super(template)
  end

  def to_s
    self.strftime("%Y-%m-%dT%H:%M:%S%:z")
  end

  def inspect
    "#<DateTime: #{self.to_s} ((#{@jd}j),(#{@fraction}),(#{@offset}))>"
  end

  def iso8601
    self.to_s
  end

  def xmlschema
    self.to_s
  end

  def rfc3339
    self.to_s
  end

  def jisx0301
    self.to_s
  end

  def httpdate
    self.new_offset(0).strftime("%a, %d %b %Y %H:%M:%S GMT")
  end

  def rfc2822
    self.strftime("%a, %-d %b %Y %H:%M:%S %z")
  end

  def rfc822
    self.rfc2822
  end

  def to_date
    Date.from_jd(@jd, @start)
  end

  def to_datetime
    self
  end

  def to_time
    parts = Date.jd_to_gregorian(@jd)
    Time.new(parts[0], parts[1], parts[2], self.hour, self.min,
             self.sec + self.sec_fraction, (@offset * 86400).to_i)
  end

  def deconstruct_keys(keys)
    all = { year: self.year, month: self.month, day: self.day, yday: self.yday,
            wday: self.wday, hour: self.hour, min: self.min, sec: self.sec,
            sec_fraction: self.sec_fraction, zone: self.zone }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end
end

class Time
  # The same instant as a DateTime, read on the calendar the offset this Time
  # carries puts it on.
  def to_datetime
    seconds = self.to_r + self.utc_offset
    days = seconds / 86400
    landing = days.floor
    DateTime.from_parts(landing + 2440588, days - landing,
                        Rational(self.utc_offset, 86400), Date::ITALY)
  end

  def to_date
    days = (self.to_r + self.utc_offset) / 86400
    Date.from_jd(days.floor + 2440588, Date::ITALY)
  end
end
