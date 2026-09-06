# Dividing two integers answers an integer, rounded toward negative infinity,
# and the modulus that pairs with it takes the sign of the divisor.
p 5 / 2
p(-5 / 2)
p 5 % 3
p(-5 % 3)
p 5.divmod 3
p(-5.divmod(3))

# `remainder` truncates that division instead, so what is left carries the
# sign of the receiver.
p 5.remainder 3
p(-5.remainder(3))

# `div` answers a whole number even for a Float divisor, `fdiv` always answers
# a Float, and `ceildiv` rounds the quotient up.
p 5.div 2.0
p 5.fdiv 2
p 4.ceildiv 3
p 4.ceildiv -3

# A pair of numbers too wide for a Float still divides to the ratio between
# them.
wide = 10 ** 400
p wide.fdiv(10 ** 399)

# An Integer refuses a zero divisor whether or not the divisor is a Float,
# where a Float receiver answers an infinity.
begin
  5 % 0.0
rescue ZeroDivisionError => error
  puts error.message
end
puts (1 / 0.0).to_s

# `Integer.sqrt` takes the whole part of a square root, and `digits` reads the
# place values of a number in a base.
p Integer.sqrt 24
p Integer.sqrt(10 ** 400) == 10 ** 200
p 12345.digits
p 1234.digits 16

# `coerce` answers the pair an operator is applied to.
p 1.coerce 2
p 1.coerce 2.5
