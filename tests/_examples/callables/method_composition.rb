# A Method composes with any callable: `>>` runs the method first and is
# strict about its arguments, `<<` runs the argument first and follows its
# strictness.
class Arithmetic
  def double(number)
    number + number
  end

  def square(number)
    number * number
  end

  def multiply(left, right)
    left * right
  end
end

double = Arithmetic.new.method :double
square = Arithmetic.new.method :square
increment = proc { |number| number + 1 }

p (double >> square).call 3
p (double << square).call 3
p (double >> increment).lambda?
p (double << increment).lambda?
p (double >> increment).is_a? Proc

multiply = Arithmetic.new.method :multiply
p (multiply >> increment).call 2, 3

callable = Object.new
def callable.call(number)
  number - 1
end
p (double >> callable).call 5

begin
  double >> Object.new
rescue TypeError => problem
  puts problem.message
end
