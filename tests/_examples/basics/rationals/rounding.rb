# A precision moves the decimal point: a positive one keeps that many places
# and answers a Rational, and zero or less answers the Integer those places
# sit in.
value = Rational 2200, 7
p value.ceil
p value.floor
p value.truncate
p value.round
p value.ceil 1
p value.floor 2
p value.round 3
p value.ceil -2
p value.floor -1

# `half:` names where a value exactly between two whole numbers goes.
p Rational(5, 2).round
p Rational(5, 2).round half: :even
p Rational(5, 2).round half: :down
p Rational(-5, 2).round half: :even

# A fraction of two numbers too wide for a Float still answers the ratio
# between them.
wide = Rational 10 ** 400, 10 ** 399
p wide.to_f

# An operand a Rational does not know is asked to coerce.
class Half
  def coerce(other)
    [other, Rational(1, 2)]
  end
end

p Rational(1, 2) + Half.new
p Rational(3, 4) <=> Half.new
