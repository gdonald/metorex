# How BigDecimal divides, reads text and Floats, and takes logarithms: a
# quotient carries the digits of its wider operand and sixteen more, divmod
# answers an Integer quotient, and BigMath works to the digits asked for.
require "bigdecimal"
require "bigdecimal/util"

one = BigDecimal "1"
three = BigDecimal "3"
p one / three
large = BigDecimal "2E55"
wide = BigDecimal "1.23456789E10"
p large / wide
p large.precision
p (BigDecimal "0.001").precision

quotient, left_over = (BigDecimal "42").divmod BigDecimal "9"
p [quotient, quotient.class, left_over]
p (BigDecimal "-7").divmod BigDecimal "Infinity"
begin
  (BigDecimal "NaN").divmod 1
rescue FloatDomainError => error
  p error.class
end

p BigDecimal "12_345.67E8_9"
p BigDecimal "1_"
begin
  BigDecimal "1__2"
rescue ArgumentError => error
  p error.message
end
p BigDecimal "invalid", exception: false
p BigDecimal 0.1
p BigDecimal 1.0 / 3, 5
p (BigDecimal -0.0).sign
p 1.5.to_d

module LoudDecimal
  def to_s(*)
    "loud"
  end
end
BigDecimal.prepend LoudDecimal
loud = BigDecimal "44.44"
p [loud.to_s, loud.inspect]

p BigMath.log (BigDecimal "2"), 30
p BigMath.exp (BigDecimal "1"), 30
ratio = Rational 1_234_567_890, 987_654_321
p BigMath.log ratio, 20

three_quarters = Rational 3, 4
p three_quarters.coerce 1.5
p three_quarters.coerce 10
quarter = Rational 1, 4
p Float quarter
