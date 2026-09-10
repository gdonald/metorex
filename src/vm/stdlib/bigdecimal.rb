# Decimal numbers with as many digits as they are given, rather than as many
# as a float has room for. A value is a sign, a string of significant digits,
# and the power of ten the digits sit against, so `0.123e3` is 123.

class BigDecimal < Numeric
  ROUND_UP = 1
  ROUND_DOWN = 2
  ROUND_HALF_UP = 3
  ROUND_HALF_DOWN = 4
  ROUND_CEILING = 5
  ROUND_FLOOR = 6
  ROUND_HALF_EVEN = 7

  ROUND_MODE = 256
  EXCEPTION_INFINITY = 1
  EXCEPTION_NaN = 2
  EXCEPTION_UNDERFLOW = 4
  EXCEPTION_OVERFLOW = 1
  EXCEPTION_ZERODIVIDE = 16
  EXCEPTION_ALL = 255

  SIGN_NaN = 0
  SIGN_POSITIVE_ZERO = 1
  SIGN_NEGATIVE_ZERO = -1
  SIGN_POSITIVE_FINITE = 2
  SIGN_NEGATIVE_FINITE = -2
  SIGN_POSITIVE_INFINITE = 3
  SIGN_NEGATIVE_INFINITE = -3

  VERSION = "3.1.4"
  BASE = 10000

  # What `BigDecimal.mode` reports for a rule that is off.
  EXCEPTION_UNDERFLOW_MODE = 4

  # How many digits a division carries when the caller names none.
  DEFAULT_PRECISION = 20

  @@rounding_mode = ROUND_HALF_UP
  @@limit = 0

  # The parts another BigDecimal reads off this one while the two are being
  # added, multiplied, or compared.
  def sign_of
    @sign
  end

  def digits_of
    @digits
  end

  def exponent_of
    @exponent
  end

  def special_of
    @special
  end

  protected :sign_of, :digits_of, :exponent_of, :special_of

  # Build one from its parts, normalising the digits so a value has one
  # spelling: no leading zero, no trailing zero, and zero has none at all.
  def self.build(sign, digits, exponent, special = nil)
    unless special.nil?
      return allocate.send :initialize_parts, sign, "", 0, special
    end
    # A leading zero is not a significant digit, and dropping one moves the
    # point along with it. A trailing zero changes nothing either way.
    without_leading = digits.sub(/\A0+/, "")
    exponent = exponent - (digits.length - without_leading.length)
    stripped = without_leading.sub(/0+\z/, "")
    if stripped.empty?
      return allocate.send :initialize_parts, sign, "", 0, nil
    end
    allocate.send :initialize_parts, sign, stripped, exponent, nil
  end

  def initialize_parts(sign, digits, exponent, special)
    @sign = sign
    @digits = digits
    @exponent = exponent
    @special = special
    self
  end

  protected :initialize_parts

  # The value read out of whatever was handed in.
  def self.interpret(value, precision = 0)
    case value
    when BigDecimal then value
    when Integer then from_integer value
    when Float then from_float value, precision
    when Rational then from_rational value, precision
    when String then from_string value
    else
      if value.respond_to? :to_str
        from_string value.to_str
      else
        raise TypeError, "can't convert #{value.class} into BigDecimal"
      end
    end
  end

  def self.from_integer(value)
    return build 1, "", 0 if value == 0
    sign = value < 0 ? -1 : 1
    digits = value.abs.to_s
    build sign, digits, digits.length
  end

  def self.from_float(value, precision = 0)
    if precision.nil? || precision == 0
      raise ArgumentError, "can't omit precision for a Float"
    end
    return build 0, "", 0, :nan if value.nan?
    if value.infinite?
      return build(value.infinite?, "", 0, :infinite)
    end
    from_string "%.#{precision}e" % value
  end

  def self.from_rational(value, precision = 0)
    if precision.nil? || precision == 0
      raise ArgumentError, "can't omit precision for a Rational"
    end
    numerator = from_integer value.numerator
    denominator = from_integer value.denominator
    numerator.div denominator, precision
  end

  # The digits a written number stands for. Anything the notation does not
  # allow is refused rather than read as far as it goes.
  def self.from_string(text)
    held = text.strip.gsub("_", "")
    return build 0, "", 0, :nan if held =~ /\A[+-]?NaN\z/
    if held =~ /\A([+-])?Infinity\z/
      return build($1 == "-" ? -1 : 1, "", 0, :infinite)
    end
    unless held =~ /\A([+-])?(\d*)(?:\.(\d*))?(?:[eEdD]([+-]?\d+))?\z/
      raise ArgumentError, "invalid value for BigDecimal(): \"#{text}\""
    end
    sign_text, whole, fraction, power = $1, $2, $3, $4
    whole = "" if whole.nil?
    fraction = "" if fraction.nil?
    if whole.empty? && fraction.empty?
      raise ArgumentError, "invalid value for BigDecimal(): \"#{text}\""
    end
    sign = sign_text == "-" ? -1 : 1
    digits = whole + fraction
    exponent = whole.length + (power.nil? ? 0 : power.to_i)
    build sign, digits, exponent
  end

  def nan?
    @special == :nan
  end

  def infinite?
    return nil unless @special == :infinite
    @sign
  end

  def finite?
    @special.nil?
  end

  def zero?
    @special.nil? && @digits.empty?
  end

  def nonzero?
    zero? ? nil : self
  end

  def sign
    return SIGN_NaN if nan?
    return @sign * 3 if @special == :infinite
    return @sign * 1 if zero?
    @sign * 2
  end

  def exponent
    zero? || !finite? ? 0 : @exponent
  end

  def precision
    finite? ? @digits.length : 0
  end

  def precs
    [precision, precision]
  end

  # `split` names the four parts a written number has: its sign, its digits,
  # the base they are written in, and the power of that base.
  def split
    return [0, "NaN", 10, 0] if nan?
    return [@sign, "Infinity", 10, 0] if @special == :infinite
    return [@sign, "0", 10, 0] if zero?
    [@sign, @digits, 10, @exponent]
  end

  def abs
    return self if @sign > 0 || nan?
    BigDecimal.build 1, @digits, @exponent, @special
  end

  def -@
    return self if nan?
    BigDecimal.build(-@sign, @digits, @exponent, @special)
  end

  def +@
    self
  end

  def coerce(other)
    held = other.is_a?(Float) ? BigDecimal.from_string(other.to_s) : BigDecimal.interpret(other, DEFAULT_PRECISION)
    [held, self]
  end

  # The other side of an operation, read as a BigDecimal, or nil when it is
  # something this class has no reading for.
  def companion(other)
    case other
    when BigDecimal then other
    when Integer then BigDecimal.from_integer(other)
    when Float then BigDecimal.from_string(other.to_s)
    when Rational then BigDecimal.from_rational(other, DEFAULT_PRECISION)
    else nil
    end
  end

  private :companion

  def +(other)
    held = companion other
    return coerced_binary(other, :+) if held.nil?
    return BigDecimal.build(0, "", 0, :nan) if nan? || held.nan?
    if !finite? || !held.finite?
      return add_infinite held
    end
    scale = [scale_of, held.scale_of].min
    total = unscaled(scale) + held.unscaled(scale)
    BigDecimal.from_scaled(total, scale).within_limit
  end

  def -(other)
    held = companion other
    return coerced_binary(other, :-) if held.nil?
    self + (-held)
  end

  def *(other)
    held = companion other
    return coerced_binary(other, :*) if held.nil?
    return BigDecimal.build(0, "", 0, :nan) if nan? || held.nan?
    if !finite? || !held.finite?
      return multiply_infinite held
    end
    sign = @sign * held.sign_of
    product = @digits.to_i * held.digits_of.to_i
    return BigDecimal.build(sign, "", 0) if product == 0
    # Each side is its digits against a power of ten, so the product's power
    # is the two added, less the digits each side spent.
    reach = product.to_s.length - @digits.length - held.digits_of.length
    BigDecimal.build(sign, product.to_s, @exponent + held.exponent_of + reach).within_limit
  end

  def div(other, digits = nil)
    held = companion other
    return coerced_binary(other, :div) if held.nil?
    if digits.nil?
      raise ZeroDivisionError, "divided by 0" if held.zero?
      quotient = quotient_with self, held, DEFAULT_PRECISION
      raise FloatDomainError, quotient.to_s("") unless quotient.finite?
      return quotient.floor_to_integer.to_integer
    end
    return quotient_with(self, held, DEFAULT_PRECISION) if digits == 0
    quotient_with self, held, digits
  end

  def /(other)
    held = companion other
    return coerced_binary(other, :/) if held.nil?
    quotient_with self, held, DEFAULT_PRECISION
  end

  def quo(other, digits = nil)
    held = companion other
    return coerced_binary(other, :quo) if held.nil?
    quotient_with self, held, digits.nil? || digits == 0 ? DEFAULT_PRECISION : digits
  end

  def add(other, digits)
    with_named_precision(digits) { self + other }
  end

  def sub(other, digits)
    with_named_precision(digits) { self - other }
  end

  def mult(other, digits)
    with_named_precision(digits) { self * other }
  end

  # A precision of its own decides on its own, and the global limit stands
  # aside. A precision of zero asks for the global limit instead.
  def with_named_precision(digits)
    unless digits.is_a? Integer
      raise TypeError, "wrong argument type #{digits.class} (expected Integer)"
    end
    if digits < 0
      raise ArgumentError, "negative precision"
    end
    return yield if digits == 0
    BigDecimal.without_limit { yield }.round_to_significant digits
  end

  private :with_named_precision

  def **(power)
    unless power.is_a? Integer
      raise TypeError, "wrong argument type #{power.class} (expected Integer)"
    end
    return BigDecimal.build(0, "", 0, :nan) if nan?
    return BigDecimal.from_string("1") if power == 0
    if power < 0
      # An inverse is carried well past the digits the value itself has, so
      # the answer settles rather than stopping at the default width.
      raised = self ** (-power)
      carried = raised.precision * 2 + 40
      carried = 120 if carried < 120
      return BigDecimal.from_string("1").quo(raised, carried)
    end
    answer = BigDecimal.from_string "1"
    power.times { answer = answer * self }
    answer
  end

  alias_method :power, :**

  def divmod(other)
    held = companion other
    return coerced_binary(other, :divmod) if held.nil?
    raise ZeroDivisionError, "divided by 0" if held.zero? && !nan? && finite?
    if nan? || held.nan? || !finite? || !held.finite?
      nan = BigDecimal.build 0, "", 0, :nan
      return [nan, nan]
    end
    quotient = whole_quotient held, false
    [quotient, self - quotient * held]
  end

  def %(other)
    held = companion other
    return coerced_binary(other, :%) if held.nil?
    raise ZeroDivisionError, "divided by 0" if held.zero? && !nan? && finite?
    divmod(held)[1]
  end

  alias_method :modulo, :%

  # `remainder` cuts the quotient toward zero, where `%` cuts it downward, so
  # the two differ in sign when the operands do.
  def remainder(other)
    held = companion other
    return coerced_binary(other, :remainder) if held.nil?
    raise ZeroDivisionError, "divided by 0" if held.zero? && !nan? && finite?
    if nan? || held.nan? || !finite? || !held.finite?
      return BigDecimal.build(0, "", 0, :nan)
    end
    quotient = whole_quotient held, true
    self - quotient * held
  end

  def <=>(other)
    held = companion other
    if held.nil? && other.respond_to?(:coerce)
      left, right = other.coerce self
      return left <=> right
    end
    return nil if held.nil?
    return nil if nan? || held.nan?
    return compare_infinite(held) if !finite? || !held.finite?
    scale = [scale_of, held.scale_of].min
    unscaled(scale) <=> held.unscaled(scale)
  end

  def ==(other)
    held = companion other
    return false if held.nil?
    return false if nan? || held.nan?
    (self <=> held) == 0
  end

  alias_method :===, :==
  alias_method :eql?, :==

  def <(other)
    ordered other, :<
  end

  def <=(other)
    ordered other, :<=
  end

  def >(other)
    ordered other, :>
  end

  def >=(other)
    ordered other, :>=
  end

  def hash
    [@sign, @digits, @exponent, @special].hash
  end

  def to_i
    raise FloatDomainError, to_s unless finite?
    truncate_to_integer.to_integer
  end

  alias_method :to_int, :to_i

  def to_f
    return Float::NAN if nan?
    return @sign * Float::INFINITY if @special == :infinite
    to_s("F").to_f
  end

  def to_r
    return nil unless finite?
    return Rational(0, 1) if zero?
    if @exponent >= @digits.length
      Rational(@sign * @digits.to_i * 10 ** (@exponent - @digits.length), 1)
    else
      Rational(@sign * @digits.to_i, 10 ** (@digits.length - @exponent))
    end
  end

  def to_d
    self
  end

  # The number written out in full, which is what `to_digits` answers.
  def to_digits
    to_s "F"
  end

  def inspect
    to_s ""
  end

  # The written form. "F" spells the number out in full, a leading "+" asks
  # for a sign on a positive value, and a number asks for the digits to be
  # grouped that many at a time.
  def to_s(format = "")
    named = format.to_s
    grouping = named[/\d+/].to_i
    plain = named.include?("F") || named.include?("f")
    leading = named.include?("+") ? "+" : (named.include?(" ") ? " " : "")
    mark = @sign < 0 ? "-" : leading
    return "#{mark}NaN" if nan?
    return "#{mark}Infinity" if @special == :infinite
    return "#{mark}0.0" if zero?
    body = plain ? plain_form(grouping) : scientific_form(grouping)
    "#{mark}#{body}"
  end

  def round(digits = 0, mode = nil)
    wanted = mode.nil? ? @@rounding_mode : BigDecimal.rounding_named(mode)
    return refuse_whole_number(digits) unless finite?
    rounded = round_at digits, wanted
    digits <= 0 ? rounded.to_integer : rounded
  end

  def ceil(digits = 0)
    return refuse_whole_number(digits) unless finite?
    rounded = round_at digits, ROUND_CEILING
    digits <= 0 ? rounded.to_integer : rounded
  end

  def floor(digits = 0)
    return refuse_whole_number(digits) unless finite?
    rounded = round_at digits, ROUND_FLOOR
    digits <= 0 ? rounded.to_integer : rounded
  end

  def truncate(digits = nil)
    # With no count at all the answer is an Integer, which a value that is
    # not a number has none of. A count names a place to cut at instead.
    return refuse_whole_number(0) if digits.nil? && !finite?
    return self unless finite?
    wanted = digits.nil? ? 0 : digits
    rounded = round_at wanted, ROUND_DOWN
    wanted <= 0 ? rounded.to_integer : rounded
  end

  # `fix` keeps the whole part and `frac` keeps what is left of it.
  def fix
    return self unless finite?
    truncate_to_integer
  end

  def frac
    return self unless finite?
    self - fix
  end

  def sqrt(digits)
    unless digits.is_a? Integer
      raise TypeError, "wrong argument type #{digits.class} (expected Integer)"
    end
    wanted_digits = digits
    if wanted_digits < 0
      raise ArgumentError, "negative precision for sqrt"
    end
    if nan? || @sign < 0 && !zero?
      raise FloatDomainError, "sqrt of negative value"
    end
    return self if !finite? || zero?
    wanted = wanted_digits
    wanted = DEFAULT_PRECISION if wanted < DEFAULT_PRECISION
    # Newton's method, carried a few digits wider than what is asked for so
    # the last digit that is kept has settled.
    working = wanted + 10
    half = BigDecimal.from_string "0.5"
    # A root sits at about half the power of ten its value does, which is a
    # near enough start that each step from there doubles the settled digits.
    guess = BigDecimal.build 1, "5", (@exponent + 1) / 2
    # Each step is carried only as wide as the digits it can settle, so the
    # early steps stay cheap and only the last one works at full width.
    carried = 16
    100.times do
      carried = carried * 2
      carried = working if carried > working
      next_guess = (guess + quotient_with(self, guess, carried)) * half
      settled = carried >= working && (next_guess - guess).zero?
      guess = next_guess
      break if settled
    end
    guess.round_to_significant wanted
  end

  # A rounding rule named by symbol or string reads back as the number it
  # stands for, so `round(2, :up)` and `round(2, ROUND_UP)` are one thing.
  NAMED_ROUNDING = {
    up: ROUND_UP,
    down: ROUND_DOWN,
    truncate: ROUND_DOWN,
    half_up: ROUND_HALF_UP,
    default: ROUND_HALF_UP,
    half_down: ROUND_HALF_DOWN,
    half_even: ROUND_HALF_EVEN,
    banker: ROUND_HALF_EVEN,
    ceiling: ROUND_CEILING,
    ceil: ROUND_CEILING,
    floor: ROUND_FLOOR
  }

  def self.rounding_named(mode)
    return mode if mode.is_a?(Integer) && (ROUND_UP..ROUND_HALF_EVEN).include?(mode)
    found = NAMED_ROUNDING[mode.to_s.downcase.to_sym] if mode.respond_to? :to_s
    raise ArgumentError, "invalid rounding mode (#{mode})" if found.nil?
    found
  end

  def self.mode(selector, value = nil)
    if selector == ROUND_MODE
      @@rounding_mode = rounding_named(value) unless value.nil?
      return @@rounding_mode
    end
    0
  end

  # Run a block with the global limit set aside, which is what an operation
  # given a precision of its own asks for.
  def self.without_limit
    held = @@limit
    @@limit = 0
    begin
      yield
    ensure
      @@limit = held
    end
  end

  def self.limit(digits = nil)
    previous = @@limit
    unless digits.nil?
      raise ArgumentError, "argument must be positive" if digits < 0
      @@limit = digits
    end
    previous
  end

  def self.double_fig
    16
  end

  def self._load(text)
    BigDecimal text.split(":").last
  end

  def _dump(_depth = nil)
    "#{precision}:#{to_s}"
  end

  # ── the parts the operations above lean on ────────────────────────────────

  # Where the last digit sits, counted as a power of ten. Two values are
  # added by bringing both to the lower of their two scales.
  def scale_of
    @exponent - @digits.length
  end

  # The digits as one integer, written against the given scale.
  def unscaled(scale)
    return 0 if @digits.empty?
    @sign * @digits.to_i * 10 ** (scale_of - scale)
  end

  protected :scale_of, :unscaled

  def self.from_scaled(total, scale)
    return build 1, "", 0 if total == 0
    sign = total < 0 ? -1 : 1
    digits = total.abs.to_s
    build sign, digits, digits.length + scale
  end

  # The quotient, carried to the number of significant digits asked for.
  def quotient_with(left, right, digits)
    return BigDecimal.build(0, "", 0, :nan) if left.nan? || right.nan?
    if !left.finite? || !right.finite?
      return divide_infinite left, right
    end
    if right.zero?
      return BigDecimal.build(0, "", 0, :nan) if left.zero?
      return BigDecimal.build(left.sign_of * right.sign_of, "", 0, :infinite)
    end
    return BigDecimal.build(left.sign_of * right.sign_of, "", 0) if left.zero?
    sign = left.sign_of * right.sign_of
    wanted = digits + 2
    shifted = left.digits_of.to_i * 10 ** (wanted + right.digits_of.length)
    quotient = shifted / right.digits_of.to_i
    exponent = left.exponent_of - right.exponent_of - wanted - left.digits_of.length
    BigDecimal.build(sign, quotient.to_s, quotient.to_s.length + exponent)
      .round_to_significant(digits)
      .within_limit
  end

  private :quotient_with

  # The same value cut to the given count of significant digits, under the
  # rounding rule in force unless another is named.
  def round_to_significant(count, mode = nil)
    return self unless finite?
    return self if zero? || count <= 0 || @digits.length <= count
    kept = @digits[0, count]
    carried = kept.to_i
    if rounds_up? @digits[count..-1], carried, mode.nil? ? @@rounding_mode : mode
      carried = carried + 1
    end
    BigDecimal.build @sign, carried.to_s, @exponent + (carried.to_s.length - count)
  end

  # A global limit caps the digits every answer carries, which is what
  # `BigDecimal.limit` sets.
  def within_limit
    return self if @@limit.nil? || @@limit.to_i < 1
    round_to_significant @@limit.to_i
  end

  # The same value with everything past `digits` places after the point
  # decided by the named rounding rule.
  def round_at(digits, mode)
    keep = @exponent + digits
    if keep <= 0 && !zero?
      # Every digit sits below the place being kept, so the answer is either
      # nothing or a single one at that place.
      dropped = ("0" * -keep) + @digits
      return BigDecimal.build(@sign, "", 0) unless rounds_up? dropped, 0, mode
      return BigDecimal.build(@sign, "1", 1 - digits)
    end
    return self if keep >= @digits.length
    kept = @digits[0, keep]
    kept = "0" if kept.empty?
    dropped = @digits[keep..-1]
    carried = kept.to_i
    carried = carried + 1 if rounds_up? dropped, carried, mode
    BigDecimal.build @sign, carried.to_s, @exponent + (carried.to_s.length - keep)
  end

  def rounds_up?(dropped, kept, mode)
    return false if dropped.nil? || dropped.empty? || dropped.to_i == 0
    first = dropped[0].to_i
    case mode
    when ROUND_UP then true
    when ROUND_DOWN then false
    when ROUND_CEILING then @sign > 0
    when ROUND_FLOOR then @sign < 0
    when ROUND_HALF_DOWN then first > 5 || (first == 5 && dropped[1..-1].to_i > 0)
    when ROUND_HALF_EVEN
      if first > 5
        true
      elsif first < 5
        false
      elsif dropped[1..-1].to_i > 0
        true
      else
        kept.odd?
      end
    else first >= 5
    end
  end

  protected :round_to_significant, :within_limit
  private :round_at, :rounds_up?

  # How many times `held` goes into this value, counted exactly rather than
  # through a quotient that carries only so many digits. `toward_zero` cuts
  # the count toward zero, which is what `remainder` asks for, and otherwise
  # it is cut downward, which is what `divmod` asks for.
  def whole_quotient(held, toward_zero)
    scale = [scale_of, held.scale_of].min
    left = unscaled scale
    right = held.unscaled scale
    if toward_zero
      magnitude = left.abs / right.abs
      sign = (left < 0) == (right < 0) ? 1 : -1
      BigDecimal.from_integer sign * magnitude
    else
      BigDecimal.from_integer left / right
    end
  end

  private :whole_quotient

  # The whole part, cut downward, still as a BigDecimal.
  def floor_to_integer
    round_at 0, ROUND_FLOOR
  end

  # The whole part, cut toward zero.
  def truncate_to_integer
    round_at 0, ROUND_DOWN
  end

  protected :floor_to_integer, :truncate_to_integer

  # The Integer a value with no fractional part stands for.
  def to_integer
    return 0 if zero?
    if @exponent >= @digits.length
      @sign * @digits.to_i * 10 ** (@exponent - @digits.length)
    else
      @sign * @digits[0, @exponent].to_i
    end
  end

  protected :to_integer

  def add_infinite(held)
    return self if finite? == false && held.finite?
    return held if finite? && !held.finite?
    if @sign == held.sign_of
      self
    else
      BigDecimal.build 0, "", 0, :nan
    end
  end

  def multiply_infinite(held)
    return BigDecimal.build(0, "", 0, :nan) if zero? || held.zero?
    BigDecimal.build @sign * held.sign_of, "", 0, :infinite
  end

  def divide_infinite(left, right)
    if !left.finite? && !right.finite?
      return BigDecimal.build(0, "", 0, :nan)
    end
    if !left.finite?
      return BigDecimal.build(left.sign_of * right.sign_of, "", 0, :infinite)
    end
    BigDecimal.build left.sign_of * right.sign_of, "", 0
  end

  def compare_infinite(held)
    left = !finite? ? @sign * 2 : (zero? ? 0 : @sign)
    right = !held.finite? ? held.sign_of * 2 : (held.zero? ? 0 : held.sign_of)
    left <=> right
  end

  private :add_infinite, :multiply_infinite, :divide_infinite, :compare_infinite

  def ordered(other, operator)
    held = companion other
    if held.nil? && other.respond_to?(:coerce)
      left, right = other.coerce self
      return left.send operator, right
    end
    return false if nan? || (!held.nil? && held.nan?)
    answer = held.nil? ? nil : (self <=> held)
    if answer.nil?
      raise ArgumentError, "comparison of BigDecimal with #{other.inspect} failed"
    end
    answer.send operator, 0
  end

  # A value with no whole number behind it has none to answer with. Asking
  # for digits after the point still answers the value itself, which is what
  # Ruby does for NaN and the infinities.
  def refuse_whole_number(digits)
    return self if digits > 0
    raise FloatDomainError, to_s("")
  end

  private :ordered, :refuse_whole_number

  # An operand this class has no reading for is asked how to be compared,
  # which is what `coerce` is for.
  def coerced_binary(other, operator)
    unless other.respond_to? :coerce
      raise TypeError, "#{other.class} can't be coerced into BigDecimal"
    end
    left, right = other.coerce self
    left.send operator, right
  end

  private :coerced_binary

  def scientific_form(grouping)
    body = grouping > 0 ? grouped(@digits, grouping) : @digits
    "0.#{body}e#{@exponent}"
  end

  def plain_form(grouping)
    if @exponent <= 0
      whole = "0"
      fraction = ("0" * -@exponent) + @digits
    elsif @exponent >= @digits.length
      whole = @digits + "0" * (@exponent - @digits.length)
      fraction = "0"
    else
      whole = @digits[0, @exponent]
      fraction = @digits[@exponent..-1]
    end
    fraction = "0" if fraction.empty?
    if grouping > 0
      whole = grouped_from_right whole, grouping
      fraction = grouped fraction, grouping
    end
    "#{whole}.#{fraction}"
  end

  private :scientific_form, :plain_form

  def grouped(text, size)
    pieces = []
    place = 0
    while place < text.length
      pieces.push text[place, size]
      place = place + size
    end
    pieces.join " "
  end

  def grouped_from_right(text, size)
    pieces = []
    rest = text
    while rest.length > size
      pieces.unshift rest[rest.length - size, size]
      rest = rest[0, rest.length - size]
    end
    pieces.unshift rest
    pieces.join " "
  end

  private :grouped, :grouped_from_right
end

# The two values that name themselves rather than a quantity. They are set
# once the class is built, since building one needs the class.
BigDecimal::INFINITY = BigDecimal.build(1, "", 0, :infinite)
BigDecimal::NAN = BigDecimal.build(0, "", 0, :nan)

# Functions over BigDecimal values that go beyond arithmetic. Each series
# below stops once its next term sits further down than the digits being
# asked for, which is the point past which it changes nothing.
module BigMath
  def self.PI(precision)
    working = precision + 10
    sixteen = BigDecimal.from_string "16"
    four = BigDecimal.from_string "4"
    # Machin's formula, whose two arctangents both converge quickly.
    first = BigMath.atan_inverse(5, working) * sixteen
    second = BigMath.atan_inverse(239, working) * four
    (first - second).round precision, BigDecimal::ROUND_HALF_UP
  end

  def self.E(precision)
    working = precision + 10
    total = BigDecimal.from_string "1"
    term = BigDecimal.from_string "1"
    step = 1
    while step < working
      term = term.div BigDecimal.from_integer(step), working
      break if settled? term, working
      total = total + term
      step = step + 1
    end
    total.round precision, BigDecimal::ROUND_HALF_UP
  end

  def self.sqrt(value, precision)
    value.sqrt precision
  end

  # `arctan(1/n)` by its series, which is what Machin's formula is built from.
  def self.atan_inverse(divisor, working)
    one = BigDecimal.from_string "1"
    term = one.div BigDecimal.from_integer(divisor), working
    square = BigDecimal.from_integer divisor * divisor
    total = term
    step = 1
    while step < working
      term = term.div square, working
      piece = term.div BigDecimal.from_integer(2 * step + 1), working
      break if settled? piece, working
      total = step.odd? ? total - piece : total + piece
      step = step + 1
    end
    total
  end

  # Whether a term has fallen below the last digit being carried, so adding
  # it would change nothing.
  def self.settled?(term, working)
    term.zero? || term.exponent < -working
  end
end

module Kernel
  def BigDecimal(value, precision = 0)
    BigDecimal.interpret value, precision
  end

  private :BigDecimal
end

class Integer
  def to_d
    BigDecimal.from_integer self
  end
end

class Float
  def to_d(precision = BigDecimal::DEFAULT_PRECISION)
    BigDecimal.from_float self, precision
  end
end

class String
  # As much of a number as the text starts with, and zero when it starts
  # with none at all.
  def to_d
    leading = self[/\A\s*[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eEdD][+-]?\d+)?/]
    return BigDecimal.from_string("0") if leading.nil?
    BigDecimal.from_string leading
  end
end

class NilClass
  def to_d
    BigDecimal.from_string "0"
  end
end

class Rational
  def to_d(precision)
    BigDecimal.from_rational self, precision
  end
end
