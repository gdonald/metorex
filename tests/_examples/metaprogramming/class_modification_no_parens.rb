# Runtime Class Modification without parentheses (where possible)
# Note: receiver.method(args) requires parens for multi-token args

class Greeter
  def hello(name)
    "Hello, #{name}!"
  end
end

Greeter.alias_method("hi", "hello")

g = Greeter.new
puts "=== alias_method ==="
puts g.hello("Alice")
puts g.hi("Bob")

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

a = Animal.new
puts ""
puts "=== remove_method ==="
puts a.move
puts a.speak

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

c = Child.new
puts ""
puts "=== undef_method ==="
puts c.farewell
puts c.greet

class StringHelper
  def shout(text)
    text.upcase
  end
end

StringHelper.alias_method("yell", "shout")
StringHelper.alias_method("scream", "shout")

sh = StringHelper.new
puts ""
puts "=== multiple aliases ==="
puts sh.shout("hello")
puts sh.yell("hello")
puts sh.scream("hello")

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

s = Sub.new
puts ""
puts "=== remove_method with inheritance ==="
puts s.greet
