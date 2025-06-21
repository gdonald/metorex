# Type System Examples
# Demonstrates the Metorex runtime object type system

# Nil type
nil_value = nil
puts "Nil: #{nil_value}"

# Boolean types
true_value = true
false_value = false
puts "Boolean true: #{true_value}"
puts "Boolean false: #{false_value}"

# Integer type
int_value = 42
puts "Integer: #{int_value}"

# Float type
float_value = 3.14159
puts "Float: #{float_value}"

# String type
string_value = "Hello, Metorex!"
puts "String: #{string_value}"

# Array type
array_value = [1, 2, 3, "four", 5.0]
puts "Array: #{array_value}"

# Dictionary/Hash type
dict_value = {
  name: "Metorex",
  version: "0.1.0",
  awesome: true
}
puts "Dictionary: #{dict_value}"

# ============================================================================
# Type Equality Comparison
# ============================================================================

puts "\n=== Equality Comparisons ==="

# Nil equality
puts "nil equals nil: #{nil == nil}"
puts "nil equals false: #{nil == false}"

# Boolean equality
puts "true equals true: #{true == true}"
puts "true equals false: #{true == false}"

# Integer equality
puts "42 equals 42: #{42 == 42}"
puts "42 equals 43: #{42 == 43}"

# Float equality
puts "3.14 equals 3.14: #{3.14 == 3.14}"
puts "3.14 equals 2.71: #{3.14 == 2.71}"

# String equality
puts "'hello' equals 'hello': #{'hello' == 'hello'}"
puts "'hello' equals 'world': #{'hello' == 'world'}"

# Array equality (deep comparison)
arr1 = [1, 2, 3]
arr2 = [1, 2, 3]
arr3 = [1, 2, 4]
puts "arr1 equals arr2: #{arr1 == arr2}"
puts "arr1 equals arr3: #{arr1 == arr3}"

# Dictionary equality (deep comparison)
dict1 = {x: 10, y: 20}
dict2 = {x: 10, y: 20}
dict3 = {x: 10, y: 30}
puts "dict1 equals dict2: #{dict1 == dict2}"
puts "dict1 equals dict3: #{dict1 == dict3}"

# ============================================================================
# Type Checking and Conversion
# ============================================================================

puts "\n=== Type Checking ==="

# Check object types
puts "42 is Integer: #{42.is_a?(Integer)}"
puts "3.14 is Float: #{3.14.is_a?(Float)}"
puts "'hello' is String: #{'hello'.is_a?(String)}"
puts "[1,2,3] is Array: #{[1,2,3].is_a?(Array)}"
puts "true is Boolean: #{true.is_a?(Boolean)}"

# String conversion
puts "\n=== String Conversion ==="
puts "Integer to string: #{42.to_s}"
puts "Float to string: #{3.14.to_s}"
puts "Boolean to string: #{true.to_s}"
puts "Array to string: #{[1, 2, 3].to_s}"

# ============================================================================
# Hashing (for use in sets and dictionaries)
# ============================================================================

puts "\n=== Hashing ==="

# Only primitive types are hashable
set_value = Set.new
set_value.add(42)
set_value.add("hello")
set_value.add(true)
set_value.add(nil)
puts "Set with primitives: #{set_value}"

# Arrays and dictionaries are not hashable
# This would raise an error:
# set_value.add([1, 2, 3])  # Error: Arrays are not hashable

# ============================================================================
# Class and Instance Types
# ============================================================================

puts "\n=== Classes and Instances ==="

class Person
  def initialize(name, age)
    @name = name
    @age = age
  end

  def to_s
    "Person(name: #{@name}, age: #{@age})"
  end

  def equals(other)
    return false unless other.is_a?(Person)
    @name == other.name && @age == other.age
  end
end

# Create instances
person1 = Person.new("Alice", 30)
person2 = Person.new("Alice", 30)
person3 = Person.new("Bob", 25)

puts "person1: #{person1}"
puts "person2: #{person2}"
puts "person3: #{person3}"

# Instance equality (reference equality by default)
puts "person1 == person2 (different references): #{person1 == person2}"
same_person = person1
puts "person1 == same_person (same reference): #{person1 == same_person}"

# Custom equality using equals method
puts "person1.equals(person2): #{person1.equals(person2)}"
puts "person1.equals(person3): #{person1.equals(person3)}"

# ============================================================================
# Result Type (for error handling)
# ============================================================================

puts "\n=== Result Type ==="

def divide(a, b)
  if b == 0
    return Result.error("Division by zero")
  end
  Result.ok(a / b)
end

result1 = divide(10, 2)
result2 = divide(10, 0)

puts "Result of 10 / 2: #{result1}"
puts "Result of 10 / 0: #{result2}"

if result1.is_ok?
  puts "Success: #{result1.unwrap}"
end

if result2.is_error?
  puts "Error: #{result2.unwrap_error}"
end

# ============================================================================
# Type Polymorphism
# ============================================================================

puts "\n=== Polymorphism ==="

def describe(obj)
  case obj
  when Integer
    "This is an integer: #{obj}"
  when Float
    "This is a float: #{obj}"
  when String
    "This is a string: #{obj}"
  when Array
    "This is an array with #{obj.length} elements"
  when nil
    "This is nil"
  else
    "Unknown type: #{obj.class}"
  end
end

puts describe(42)
puts describe(3.14)
puts describe("hello")
puts describe([1, 2, 3])
puts describe(nil)
puts describe(person1)
