# An operand a number does not know is asked to `coerce`, which answers the
# pair the operator is applied to instead.
class Scaled
  def initialize(value)
    @value = value
  end

  def coerce(other)
    [other, @value]
  end
end

three = Scaled.new(3)
p(6 & three)
p(6 | three)
p(6 ^ three)
p(6 * three)
p(6 - three)
p(6 > three)
p(6 <=> three)

# An error raised inside `coerce` belongs to the caller.
class Refuses
  def coerce(other)
    raise(ArgumentError, "will not coerce")
  end
end

begin
  1 + Refuses.new
rescue ArgumentError => error
  puts(error.message)
end

# A number past the Float range compares exactly, rather than by rounding to
# a Float first.
huge = (2 ** 64) + 38
p(huge <= (huge + 0.0))
p(huge > (huge + 0.0))
p(huge.gcd(24))
p(huge.lcm(1) == huge)
