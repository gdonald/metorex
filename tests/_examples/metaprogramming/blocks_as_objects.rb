# Blocks as First-Class Objects in Metorex
# Demonstrates that blocks can be assigned, passed, returned, and manipulated like any other value

puts "=== Blocks as First-Class Objects ==="
puts ""

# Example 1: Assigning blocks to variables
puts "1. Assigning blocks to variables:"
double = lambda do |x| x * 2 end
result = double.call(5)
puts "double.call(5) = #{result}"
puts ""

# Example 2: Multiple parameter blocks
puts "2. Multiple parameter blocks:"
add = lambda do |a, b| a + b end
sum = add.call(3, 7)
puts "add.call(3, 7) = #{sum}"
puts ""

# Example 3: Passing blocks as arguments
puts "3. Passing blocks as arguments to functions:"
def apply_twice(func, value)
  result = func.call(value)
  func.call(result)
end

increment = lambda do |n| n + 1 end
final = apply_twice(increment, 5)
puts "apply_twice(increment, 5) = #{final}"
puts ""

# Example 4: Returning blocks from functions
puts "4. Returning blocks from functions (closures):"
def make_multiplier(factor)
  lambda do |x| x * factor end
end

times_three = make_multiplier(3)
times_ten = make_multiplier(10)

result3 = times_three.call(4)
result10 = times_ten.call(4)

puts "times_three.call(4) = #{result3}"
puts "times_ten.call(4) = #{result10}"
puts ""

# Example 5: Blocks capturing variables (closures)
puts "5. Blocks capturing variables from outer scope:"
counter_value = 0

increment_counter = lambda do ||
  counter_value = counter_value + 1
  counter_value
end

puts "First call: #{increment_counter.call}"
puts "Second call: #{increment_counter.call}"
puts "Third call: #{increment_counter.call}"
puts ""

# Example 6: Partial application pattern
puts "6. Partial application pattern:"
def make_greeter(greeting)
  lambda do |name| "#{greeting}, #{name}!" end
end

say_hello = make_greeter("Hello")
say_goodbye = make_greeter("Goodbye")

puts say_hello.call("Alice")
puts say_goodbye.call("Bob")
puts ""

puts "=== Blocks are truly first-class objects! ==="
