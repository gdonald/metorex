# Inheritance Example
# Demonstrates class inheritance, method lookup, and method overriding

# Base class
class Animal
  def initialize(name)
    @name = name
  end

  def get_name()
    @name
  end

  def speak()
    "Some generic sound"
  end

  def introduce()
    @name + " says: " + speak()
  end
end

# Dog inherits from Animal and overrides speak
class Dog < Animal
  def speak()
    "Woof!"
  end
end

# Cat inherits from Animal and overrides speak
class Cat < Animal
  def speak()
    "Meow!"
  end
end

# Create instances
dog = Dog("Buddy")
cat = Cat("Whiskers")

# Test method lookup - get_name and introduce are inherited from Animal
puts "Dog's name: #{dog.get_name()}"
puts "Cat's name: #{cat.get_name()}"

# Test method overriding - speak is overridden in Dog and Cat
puts dog.introduce()  # "Buddy says: Woof!"
puts cat.introduce()  # "Whiskers says: Meow!"

# Multi-level inheritance example
class Bird < Animal
  def initialize(name, can_fly)
    @name = name
    @can_fly = can_fly
  end

  def speak()
    "Tweet!"
  end

  def can_fly()
    @can_fly
  end
end

class Penguin < Bird
  def initialize(name)
    @name = name
    @can_fly = false
  end

  def speak()
    "Honk!"
  end
end

# Create bird instances
parrot = Bird("Polly", true)
penguin = Penguin("Pingu")

puts parrot.introduce()   # "Polly says: Tweet!"
puts penguin.introduce()  # "Pingu says: Honk!"

if penguin.can_fly()
  puts "Penguins can fly!"
else
  puts "Penguins cannot fly"
end

# Diamond inheritance is not allowed (single inheritance only)
# but we can demonstrate inheritance chains
class GrandParent
  def method_a()
    "From GrandParent"
  end
end

class Parent < GrandParent
  def method_b()
    "From Parent"
  end
end

class Child < Parent
  def method_c()
    "From Child"
  end
end

c = Child()
puts c.method_a()  # Inherited from GrandParent
puts c.method_b()  # Inherited from Parent
puts c.method_c()  # Defined in Child

# Class variables are shared across inheritance hierarchy
class Counter
  @@total = 0

  def initialize()
    @@total = @@total + 1
    @id = @@total
  end

  def get_id()
    @id
  end

  def get_total()
    @@total
  end
end

class SpecialCounter < Counter
end

c1 = Counter()
c2 = SpecialCounter()
c3 = Counter()

puts "Counter 1 ID: #{c1.get_id()}"
puts "Counter 2 ID: #{c2.get_id()}"
puts "Counter 3 ID: #{c3.get_id()}"
puts "Total counters: #{c1.get_total()}"
