# REPL Demo - Interactive examples
# This file demonstrates code that works well in the REPL

# Simple arithmetic
puts 1 + 2

# Variable assignment
x = 42
puts "x is #{x}"

# String operations
name = "Metorex"
greeting = "Hello, #{name}!"
puts greeting

# Array operations
numbers = [1, 2, 3, 4, 5]
puts "Array length: #{numbers.length}"

# Hash operations
person = {"name" => "Alice", "age" => 30}
puts "Name: #{person["name"]}"

# Function definition
def square(n)
  n * n
end

puts "Square of 5: #{square(5)}"

# Lambda
double = lambda do |x|
  x * 2
end

puts "Double of 10: #{double.call(10)}"

# Range
range = 1..5
puts "Range to array: #{range.to_a}"
