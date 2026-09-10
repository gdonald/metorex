# A cursor over a string that matches patterns at the place it has reached,
# which is how a hand-written parser reads its input.
class StringScanner
  Error = Class.new(StandardError)
  Version = "3.1.0"

  def initialize(string, fixed_anchor: false)
    self.string = string
    @fixed_anchor = fixed_anchor
  end

  def string
    @string
  end

  def string=(text)
    converted = text.is_a?(String) ? text : text.to_str
    # A String subclass is copied into a plain String, which is what every
    # piece the scanner hands back is.
    @string = if converted.instance_of?(String)
      converted
    else
      converted + ""
    end
    reset
    text
  end

  private :initialize

  def fixed_anchor?
    @fixed_anchor ? true : false
  end

  def concat(text)
    @string = @string + text
    self
  end

  def <<(text)
    concat(text)
  end

  # ── Where the cursor stands ──────────────────────────────────────────────

  # The cursor is reported in bytes, so a multi-byte character counts for
  # every byte it is made of.
  def pos
    @string[0, @position].bytesize
  end

  def pointer
    pos
  end

  def pos=(offset)
    counted = @string.bytesize
    landing = offset < 0 ? offset + counted : offset
    if landing < 0 || landing > counted
      raise RangeError, "index out of range"
    end
    @position = characters_before landing
    offset
  end

  # The number of characters standing before a byte offset.
  def characters_before(counted)
    at = 0
    seen = 0
    while seen < counted && at < @string.length
      seen += @string[at].bytesize
      at += 1
    end
    at
  end
  private :characters_before

  def pointer=(offset)
    self.pos = offset
  end

  def charpos
    @string[0, @position].length
  end

  def reset
    @position = 0
    forget_match
    self
  end

  def terminate
    @position = @string.length
    forget_match
    self
  end

  def clear
    terminate
  end

  def rest
    @string[@position..-1] || ""
  end

  def rest?
    !eos?
  end

  def rest_size
    rest.length
  end

  def restsize
    rest_size
  end

  def eos?
    @position >= @string.length
  end

  def empty?
    eos?
  end

  def beginning_of_line?
    return true if @position == 0
    return false if @position > @string.length
    @string[@position - 1] == "\n"
  end

  def bol?
    beginning_of_line?
  end

  # ── Matching ─────────────────────────────────────────────────────────────

  # The pattern a call was given, as a Regexp. A String stands for itself
  # rather than for what it would mean as a pattern.
  def as_pattern(pattern)
    return pattern if pattern.is_a?(Regexp)
    if pattern.is_a?(String)
      return Regexp.new(Regexp.escape(pattern))
    end
    raise TypeError, "wrong argument type #{pattern.class} (expected Regexp)"
  end
  private :as_pattern

  def forget_match
    @match = nil
    @match_start = nil
    @match_text = nil
    @previous = nil
  end
  private :forget_match

  # Match the pattern against what is left. `anchored` requires the match to
  # begin where the cursor stands.
  # The pattern a searching call was given. Unlike the ones that match where
  # the cursor stands, these take a Regexp and nothing else.
  def as_searched_pattern(pattern)
    return pattern if pattern.is_a?(Regexp)
    raise TypeError, "wrong argument type #{pattern.class} (expected Regexp)"
  end
  private :as_searched_pattern

  def attempt(pattern, anchored)
    compiled = anchored ? as_pattern(pattern) : as_searched_pattern(pattern)
    remaining = rest
    found = compiled.match(remaining)
    if found.nil? || (anchored && found.begin(0) != 0)
      @match = nil
      @match_start = nil
      @match_text = nil
      return nil
    end
    @match = found
    @match_start = @position + found.begin(0)
    @match_text = found[0]
    found
  end
  private :attempt

  def scan(pattern)
    found = attempt(pattern, true)
    return nil if found.nil?
    @previous = @position
    @position = @position + @match_text.length
    @match_text
  end

  def check(pattern)
    found = attempt(pattern, true)
    found.nil? ? nil : @match_text
  end

  def match?(pattern)
    found = attempt(pattern, true)
    found.nil? ? nil : @match_text.length
  end

  def skip(pattern)
    found = attempt(pattern, true)
    return nil if found.nil?
    @previous = @position
    @position = @position + @match_text.length
    @match_text.length
  end

  def scan_until(pattern)
    found = attempt(pattern, false)
    return nil if found.nil?
    @previous = @position
    taken = @string[@position, @match_start + @match_text.length - @position]
    @position = @match_start + @match_text.length
    taken
  end

  def check_until(pattern)
    found = attempt(pattern, false)
    return nil if found.nil?
    @string[@position, @match_start + @match_text.length - @position]
  end

  def skip_until(pattern)
    taken = scan_until(pattern)
    taken.nil? ? nil : taken.length
  end

  def exist?(pattern)
    found = attempt(pattern, false)
    return nil if found.nil?
    @match_start + @match_text.length - @position
  end

  def search_full(pattern, advance, return_string)
    found = attempt(pattern, false)
    return nil if found.nil?
    reach = @match_start + @match_text.length - @position
    taken = @string[@position, reach]
    if advance
      @previous = @position
      @position = @position + reach
    end
    return_string ? taken : reach
  end

  def scan_full(pattern, advance, return_string)
    found = attempt(pattern, true)
    return nil if found.nil?
    reach = @match_text.length
    taken = @match_text
    if advance
      @previous = @position
      @position = @position + reach
    end
    return_string ? taken : reach
  end

  def unscan
    raise Error, "unscan failed: previous match record not exist" if @previous.nil?
    @position = @previous
    forget_match
    self
  end

  # ── What the last match found ────────────────────────────────────────────

  def matched
    @match_text
  end

  def matched?
    !@match.nil?
  end

  def matched_size
    @match_text.nil? ? nil : @match_text.length
  end

  def pre_match
    return nil if @match.nil?
    @string[0, @match_start]
  end

  def post_match
    return nil if @match.nil?
    @string[@match_start + @match_text.length..-1] || ""
  end

  def [](index)
    if index.is_a?(Range)
      raise TypeError, "no implicit conversion of Range into Integer"
    end
    return nil if @match.nil?
    @match[index]
  end

  def captures
    return nil if @match.nil?
    @match.captures
  end

  def named_captures
    return {} if @match.nil?
    @match.named_captures
  end

  def values_at(*indexes)
    return nil if @match.nil?
    indexes.map { |index| @match[index] }
  end

  def size
    return nil if @match.nil?
    @match.captures.length + 1
  end

  # ── Reading a piece at a time ────────────────────────────────────────────

  def getch
    return nil if eos?
    @previous = @position
    letter = @string[@position]
    @match_start = @position
    @match_text = letter
    @match = letter.match(/\A./m)
    @position = @position + 1
    letter
  end

  def get_byte
    getch
  end

  def scan_byte
    letter = getch
    letter.nil? ? nil : letter.bytes[0]
  end

  def peek(length)
    raise ArgumentError, "negative string size (or size too big)" if length < 0
    @string[@position, length] || ""
  end

  def peep(length)
    peek(length)
  end

  def peek_byte
    return nil if eos?
    @string[@position].bytes[0]
  end

  # The number a scanner reads off the front, with an optional sign and an
  # optional base marker.
  def scan_integer(base: 10)
    unless base == 10 || base == 16
      raise ArgumentError, "invalid radix #{base}"
    end
    pattern = base == 16 ? /\A[+-]?(0[xX])?\h+/ : /\A[+-]?\d+/
    found = scan(pattern)
    return nil if found.nil?
    base == 16 ? found.gsub(/0[xX]/, "").to_i(16) : found.to_i
  end

  def inspect
    return "#<StringScanner fin>" if eos?
    written = "#<StringScanner #{@position}/#{@string.length}"
    unless @position == 0
      before = @string[0, @position]
      before = before[before.length - 5, 5] if before.length > 5
      written += " #{trimmed(before, true)}"
    end
    "#{written} @ #{trimmed(rest, false)}>"
  end

  # A window on the string, written out with an ellipsis where it was cut.
  def trimmed(text, at_start)
    return text.inspect if text.length <= 5
    if at_start
      "...#{text[text.length - 5, 5]}".inspect
    else
      "#{text[0, 5]}...".inspect
    end
  end
  private :trimmed

  def must_C_version
    self
  end
end

# Raised when a scanner is asked to undo a scan it never made.
ScanError = StringScanner::Error
