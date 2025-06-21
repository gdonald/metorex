# Lambda parsing tests
# Single parameter arrow lambda
add_ten = x -> x + 10
puts add_ten.call(0)

# Multi-parameter arrow lambda
add = (x, y) -> x + y
puts add.call(5, 5)

# Lambda do/end syntax
multiply = lambda do |x, y| x * y end
puts multiply.call(6, 7)

# Nested lambdas
outer = lambda do |x|
  lambda do |y| x + y end
end
inner = outer.call(10)
puts inner.call(20)

# Lambda in array
func1 = x -> x + 10
func2 = x -> x - 5
func3 = x -> x * 2
funcs = [func1, func2, func3]
puts funcs[0].call(3)
puts funcs[1].call(18)
puts funcs[2].call(9)

# Lambda with closure
base = 5
add_base = x -> x + base
puts add_base.call(6)
puts add_base.call(9)

# Chained calls
double = x -> x * 2
add_one = x -> x + 1
puts add_one.call(double.call(10))
puts double.call(add_one.call(11))
puts add_one.call(add_one.call(8))
