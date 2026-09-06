# The sign predicates, and the magnitude `abs` also answers.
p 0.0.zero?
p 1.5.positive?
p(-1.5.negative?)
p(-2.5.magnitude)

# `angle` is the direction the number points: zero for a positive one and Pi
# for a negative one, which a signed zero follows.
p 1.0.angle
p(-1.0.angle == Math::PI)
p(-0.0.angle == Math::PI)

# The neighboring representable numbers, a single step away.
step = 0.0.next_float
p step > 0.0
p step.prev_float
p Float::INFINITY.next_float

# `coerce` answers the pair an operator is applied to, reading a String the
# way `Float()` reads one.
p 1.2.coerce 1
p 1.0.coerce "2.5"

# The parts of the exact fraction the number stands for.
p 0.5.numerator
p 0.5.denominator
p 12.0.to_r

# A precision keeps that many digits after the point, and zero or less answers
# the whole number those digits sit in.
p 3.14.floor 1
p 3.14.ceil 1
p 34.56.truncate 1
p 1234.5.floor -2

# Nothing compares against NaN, and `eql?` is equality without conversion.
nan = 0 / 0.0
p nan > 1.0
p 1.0.eql? 1
p 1.0.eql? 1.0

# `%` refuses a zero divisor whatever the receiver, where `/` answers an
# infinity.
begin
  1.0 % 0.0
rescue ZeroDivisionError => error
  puts error.message
end
p 7.0.divmod 3
p 7.0.fdiv 2
