# Math's functions are module functions, so both `Math.sqrt` and a private
# `sqrt` inside a class that includes Math reach the same one.
p(Math.sqrt(4))
p(Math.cbrt(27))
p(Math.hypot(3, 4))
p(Math.log(1))
p(Math.log(8, 2))
p(Math.log2(8))
p(Math.log10(1000))
p(Math.exp(0))
p(Math.frexp(1234))
p(Math.ldexp(0.6025390625, 11))

class Distance
  include Math

  def initialize(x, y)
    @x = x
    @y = y
  end

  def length
    sqrt(@x * @x + @y * @y)
  end
end

p(Distance.new(3, 4).length)
p(Math.private_instance_methods.include?(:sqrt))

# A logarithm of a number too wide for a Float keeps the digits the Float
# cannot hold.
p(Math.log2(2 ** 10001))

# An argument outside a function's domain, and one that is not a number at
# all, are each refused.
begin
  Math.sqrt(-1)
rescue Math::DomainError => error
  puts(error.message)
end

begin
  Math.sqrt("four")
rescue TypeError => error
  puts(error.message)
end

p(Math.sqrt(0.0 / 0.0).nan?)
