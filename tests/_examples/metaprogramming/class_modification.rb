# Runtime Class Modification
# Demonstrates remove_method, undef_method, alias_method, module_function

# 1. alias_method — create an alias for an existing method
class Greeter
  def hello(name)
    "Hello, #{name}!"
  end
end

Greeter.alias_method("hi", "hello")

g = Greeter.new()
puts "=== alias_method ==="
puts g.hello("Alice")
puts g.hi("Bob")

# 2. remove_method — remove a method from a class
class Animal
  def speak
    "..."
  end

  def move
    "moving"
  end

  def method_missing(name)
    "no method: #{name}"
  end
end

Animal.remove_method("speak")

a = Animal.new()
puts ""
puts "=== remove_method ==="
puts a.move()
puts a.speak()

# 3. undef_method — prevent method from being called, even if inherited
class Base
  def greet
    "Base greet"
  end

  def farewell
    "Base farewell"
  end
end

class Child < Base
  def greet
    "Child greet"
  end

  def method_missing(name)
    "undefined: #{name}"
  end
end

Child.undef_method("greet")

c = Child.new()
puts ""
puts "=== undef_method ==="
puts c.farewell()
puts c.greet()

# 4. alias_method with multiple aliases
class StringHelper
  def shout(text)
    text.upcase
  end
end

StringHelper.alias_method("yell", "shout")
StringHelper.alias_method("scream", "shout")

sh = StringHelper.new()
puts ""
puts "=== multiple aliases ==="
puts sh.shout("hello")
puts sh.yell("hello")
puts sh.scream("hello")

# 5. module_function — make module method callable on module itself
module MathHelper
  def double(x)
    x * 2
  end

  def triple(x)
    x * 3
  end
end

MathHelper.module_function("double")
MathHelper.module_function("triple")

puts ""
puts "=== module_function ==="
puts MathHelper.double(7)
puts MathHelper.triple(4)

# 6. remove_method only affects the class, not subclasses
class Parent
  def greet
    "Parent greet"
  end
end

class Sub < Parent
  def greet
    "Sub greet"
  end
end

Sub.remove_method("greet")

s = Sub.new()
puts ""
puts "=== remove_method with inheritance ==="
puts s.greet()
