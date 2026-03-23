# define_method - dynamically define methods on a class

# Define a method on a class using define_method called on the class
class Greeter
end

Greeter.define_method(:hello) { |name| "Hello, #{name}!" }

g = Greeter.new
puts g.hello("World")

# define_method inside a class body
class Calculator
  define_method(:double) { |n| n * 2 }
  define_method(:triple) { |n| n * 3 }
end

calc = Calculator.new
puts calc.double(5)
puts calc.triple(4)

# define_method with no parameters
class Greeter2
  define_method(:greet) { "Hi there!" }
end

g2 = Greeter2.new
puts g2.greet

# Dynamically define methods using a loop
class NumberMethods
end

["zero", "one", "two"].each do |word|
  NumberMethods.define_method(word) { word }
end

nm = NumberMethods.new
puts nm.zero
puts nm.one
puts nm.two

# respond_to? reflects dynamically defined methods
puts Greeter.new.respond_to?("hello")
