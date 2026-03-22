# Built-in Classes Examples
# Demonstrates the Metorex built-in class hierarchy and methods

# ============================================================================
# Object Class - Base class for all objects
# ============================================================================

puts "=== Object Class ==="

# Every object has access to Object methods
x = 42
puts "x.class: #{x.class}"           # Integer
puts "x.to_s: #{x.to_s}"             # "42"
puts "x.respond_to?('to_s'): #{x.respond_to?('to_s')}"  # true

y = "hello"
puts "y.class: #{y.class}"           # String
puts "y.to_s: #{y.to_s}"             # "hello"

# ============================================================================
# String Class
# ============================================================================

puts "\n=== String Class ==="

str = "Hello, World!"
puts "Original: #{str}"
puts "Length: #{str.length}"
puts "Uppercase: #{str.upcase}"
puts "Lowercase: #{str.downcase}"

# String concatenation
greeting = "Hello"
name = "Alice"
message = greeting + ", " + name + "!"
puts "Concatenated: #{message}"

# ============================================================================
# Integer Class
# ============================================================================

puts "\n=== Integer Class ==="

num1 = 42
num2 = 10

puts "num1: #{num1}"
puts "num2: #{num2}"
puts "num1.class: #{num1.class}"

# Integer operations (will be implemented in interpreter)
# puts "Addition: #{num1 + num2}"
# puts "Subtraction: #{num1 - num2}"
# puts "Multiplication: #{num1 * num2}"
# puts "Division: #{num1 / num2}"

# ============================================================================
# Float Class
# ============================================================================

puts "\n=== Float Class ==="

pi = 3.14159
e = 2.71828

puts "pi: #{pi}"
puts "e: #{e}"
puts "pi.class: #{pi.class}"

# Float operations
# puts "Sum: #{pi + e}"
# puts "Difference: #{pi - e}"

# ============================================================================
# Array Class
# ============================================================================

puts "\n=== Array Class ==="

arr = [1, 2, 3, 4, 5]
puts "Array: #{arr}"
puts "Length: #{arr.length}"

# Array operations
arr.push(6)
puts "After push(6): #{arr}"

last = arr.pop
puts "Popped: #{last}"
puts "After pop: #{arr}"

# Array indexing
puts "First element: #{arr[0]}"
puts "Third element: #{arr[2]}"

# Mixed type arrays
mixed = [1, "two", 3.0, true, nil]
puts "Mixed array: #{mixed}"

# ============================================================================
# Hash/Dictionary Class
# ============================================================================

puts "\n=== Hash Class ==="

person = {
  name: "Bob",
  age: 30,
  city: "New York"
}

puts "Hash: #{person}"
puts "Name: #{person[:name]}"
puts "Age: #{person[:age]}"

# Adding new key-value pair
person[:job] = "Engineer"
puts "After adding job: #{person}"

# ============================================================================
# Set Class
# ============================================================================

puts "\n=== Set Class ==="

numbers = Set.new
numbers.add(1)
numbers.add(2)
numbers.add(3)
numbers.add(2)  # Duplicate, will be ignored

puts "Set: #{numbers}"
puts "Contains 2: #{numbers.include?(2)}"
puts "Contains 5: #{numbers.include?(5)}"

# ============================================================================
# Exception Hierarchy
# ============================================================================

puts "\n=== Exception Hierarchy ==="

# Base Exception class
begin
  raise Exception.new("Generic exception")
rescue Exception => e
  puts "Caught Exception: #{e.message}"
end

# StandardError (most common exception type)
begin
  raise StandardError.new("Standard error occurred")
rescue StandardError => e
  puts "Caught StandardError: #{e.message}"
end

# RuntimeError (most common runtime exception)
begin
  raise RuntimeError.new("Runtime error occurred")
rescue RuntimeError => e
  puts "Caught RuntimeError: #{e.message}"
end

# TypeError (type-related errors)
begin
  raise TypeError.new("Expected Integer, got String")
rescue TypeError => e
  puts "Caught TypeError: #{e.message}"
end

# ValueError (value-related errors)
begin
  raise ValueError.new("Invalid value provided")
rescue ValueError => e
  puts "Caught ValueError: #{e.message}"
end

# ============================================================================
# Exception Hierarchy Demonstration
# ============================================================================

puts "\n=== Exception Catching ==="

# Catching specific exceptions
def divide(a, b)
  if b == 0
    raise ValueError.new("Division by zero")
  end
  a / b
end

begin
  result = divide(10, 0)
rescue ValueError => e
  puts "Value error: #{e.message}"
rescue StandardError => e
  puts "Standard error: #{e.message}"
end

# Catching parent exception catches child exceptions
begin
  raise RuntimeError.new("Runtime problem")
rescue StandardError => e
  puts "Caught via StandardError: #{e.message}"
end

# ============================================================================
# Class Hierarchy Introspection
# ============================================================================

puts "\n=== Class Hierarchy ==="

puts "Integer is a subclass of Object: #{Integer < Object}"
puts "String is a subclass of Object: #{String < Object}"
puts "RuntimeError is a subclass of StandardError: #{RuntimeError < StandardError}"
puts "StandardError is a subclass of Exception: #{StandardError < Exception}"
puts "Exception is a subclass of Object: #{Exception < Object}"

# ============================================================================
# Method Availability Through Inheritance
# ============================================================================

puts "\n=== Inherited Methods ==="

# All objects inherit from Object class
arr = [1, 2, 3]
str = "test"
num = 42

puts "Array has to_s: #{arr.respond_to?('to_s')}"
puts "String has to_s: #{str.respond_to?('to_s')}"
puts "Integer has to_s: #{num.respond_to?('to_s')}"

# ============================================================================
# Custom Classes Inherit from Object
# ============================================================================

puts "\n=== Custom Classes ==="

class Person
  def initialize(name, age)
    @name = name
    @age = age
  end

  def to_s
    "Person(name: #{@name}, age: #{@age})"
  end
end

person = Person.new("Charlie", 25)
puts "Person instance: #{person}"
puts "Person class: #{person.class}"
puts "Person has to_s: #{person.respond_to?('to_s')}"

# Person inherits from Object
puts "Person is a subclass of Object: #{Person < Object}"

# ============================================================================
# Polymorphism Through Built-in Classes
# ============================================================================

puts "\n=== Polymorphism ==="

def describe_value(value)
  case value.class
  when Integer
    "An integer: #{value}"
  when Float
    "A float: #{value}"
  when String
    "A string: #{value}"
  when Array
    "An array with #{value.length} elements"
  else
    "Unknown type: #{value.class}"
  end
end

puts describe_value(42)
puts describe_value(3.14)
puts describe_value("hello")
puts describe_value([1, 2, 3, 4, 5])

# ============================================================================
# Type Checking
# ============================================================================

puts "\n=== Type Checking ==="

value = 42

puts "Is Integer: #{value.is_a?(Integer)}"
puts "Is Float: #{value.is_a?(Float)}"
puts "Is Object: #{value.is_a?(Object)}"
puts "Is String: #{value.is_a?(String)}"

# Array type checking
arr = [1, 2, 3]
puts "Array is Array: #{arr.is_a?(Array)}"
puts "Array is Object: #{arr.is_a?(Object)}"
