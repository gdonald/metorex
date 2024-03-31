# Instance Structure Examples
# Demonstrates Metorex runtime instance creation, variables, and method dispatch

# ============================================================================
# Basic Instance Creation
# ============================================================================

puts "=== Basic Instance Creation ==="

class Person
  def initialize(name, age)
    @name = name
    @age = age
  end

  def greet
    "Hello, my name is #{@name}"
  end

  def info
    "#{@name} is #{@age} years old"
  end
end

# Create an instance
person = Person.new("Alice", 30)
puts person.greet
puts person.info

# ============================================================================
# Instance Variables
# ============================================================================

puts "\n=== Instance Variables ==="

class Counter
  def initialize
    @count = 0
  end

  def increment
    @count = @count + 1
  end

  def decrement
    @count = @count - 1
  end

  def value
    @count
  end

  def reset
    @count = 0
  end
end

counter = Counter.new
puts "Initial value: #{counter.value}"

counter.increment
puts "After increment: #{counter.value}"

counter.increment
counter.increment
puts "After two more increments: #{counter.value}"

counter.decrement
puts "After decrement: #{counter.value}"

counter.reset
puts "After reset: #{counter.value}"

# ============================================================================
# Multiple Instance Variables
# ============================================================================

puts "\n=== Multiple Instance Variables ==="

class Rectangle
  def initialize(width, height)
    @width = width
    @height = height
  end

  def area
    @width * @height
  end

  def perimeter
    2 * (@width + @height)
  end

  def scale(factor)
    @width = @width * factor
    @height = @height * factor
  end

  def dimensions
    "#{@width} x #{@height}"
  end
end

rect = Rectangle.new(10, 5)
puts "Rectangle dimensions: #{rect.dimensions}"
puts "Area: #{rect.area}"
puts "Perimeter: #{rect.perimeter}"

rect.scale(2)
puts "After scaling by 2: #{rect.dimensions}"
puts "New area: #{rect.area}"

# ============================================================================
# Instance Variable Types
# ============================================================================

puts "\n=== Different Variable Types ==="

class DataHolder
  def initialize
    @string_var = "hello"
    @int_var = 42
    @float_var = 3.14
    @bool_var = true
    @nil_var = nil
    @array_var = [1, 2, 3]
    @dict_var = {key: "value"}
  end

  def show_string
    "String: #{@string_var}"
  end

  def show_int
    "Integer: #{@int_var}"
  end

  def show_float
    "Float: #{@float_var}"
  end

  def show_bool
    "Boolean: #{@bool_var}"
  end

  def show_nil
    "Nil: #{@nil_var}"
  end

  def show_array
    "Array: #{@array_var}"
  end

  def show_dict
    "Dict: #{@dict_var}"
  end
end

holder = DataHolder.new
puts holder.show_string
puts holder.show_int
puts holder.show_float
puts holder.show_bool
puts holder.show_nil
puts holder.show_array
puts holder.show_dict

# ============================================================================
# Method Dispatch with Inheritance
# ============================================================================

puts "\n=== Method Dispatch with Inheritance ==="

class Animal
  def initialize(name)
    @name = name
  end

  def speak
    "#{@name} makes a sound"
  end

  def move
    "#{@name} moves"
  end
end

class Dog < Animal
  def initialize(name, breed)
    super(name)
    @breed = breed
  end

  def speak
    "#{@name} barks"
  end

  def fetch
    "#{@name} fetches the ball"
  end
end

animal = Animal.new("Generic")
puts animal.speak
puts animal.move

dog = Dog.new("Buddy", "Golden Retriever")
puts dog.speak         # Calls Dog's speak (overridden)
puts dog.move          # Calls Animal's move (inherited)
puts dog.fetch         # Calls Dog's fetch (new method)

# ============================================================================
# Instance State Mutation
# ============================================================================

puts "\n=== Instance State Mutation ==="

class BankAccount
  def initialize(holder, balance)
    @holder = holder
    @balance = balance
  end

  def deposit(amount)
    @balance = @balance + amount
    "Deposited #{amount}. New balance: #{@balance}"
  end

  def withdraw(amount)
    if amount > @balance
      "Insufficient funds. Balance: #{@balance}"
    else
      @balance = @balance - amount
      "Withdrew #{amount}. New balance: #{@balance}"
    end
  end

  def balance
    @balance
  end
end

account = BankAccount.new("Alice", 1000)
puts "Initial balance: #{account.balance}"

puts account.deposit(500)
puts account.withdraw(300)
puts account.withdraw(2000)

# ============================================================================
# Multiple Instances
# ============================================================================

puts "\n=== Multiple Instances ==="

class Point
  def initialize(x, y)
    @x = x
    @y = y
  end

  def to_s
    "(#{@x}, #{@y})"
  end

  def distance_to(other)
    dx = @x - other.x
    dy = @y - other.y
    Math.sqrt(dx * dx + dy * dy)
  end

  def x
    @x
  end

  def y
    @y
  end
end

p1 = Point.new(0, 0)
p2 = Point.new(3, 4)
p3 = Point.new(1, 1)

puts "Point 1: #{p1.to_s}"
puts "Point 2: #{p2.to_s}"
puts "Point 3: #{p3.to_s}"

puts "Distance from p1 to p2: #{p1.distance_to(p2)}"
puts "Distance from p2 to p3: #{p2.distance_to(p3)}"

# ============================================================================
# Instance Methods Accessing Other Instance Methods
# ============================================================================

puts "\n=== Methods Calling Other Methods ==="

class Circle
  def initialize(radius)
    @radius = radius
  end

  def radius
    @radius
  end

  def diameter
    @radius * 2
  end

  def circumference
    2 * Math::PI * @radius
  end

  def area
    Math::PI * @radius * @radius
  end

  def describe
    "Circle with radius #{radius}, diameter #{diameter}, " +
    "circumference #{circumference.round(2)}, " +
    "and area #{area.round(2)}"
  end
end

circle = Circle.new(5)
puts circle.describe

# ============================================================================
# Class Name Introspection
# ============================================================================

puts "\n=== Class Name Introspection ==="

class Vehicle
  def initialize(make, model)
    @make = make
    @model = model
  end

  def identify
    "This is a #{self.class} instance: #{@make} #{@model}"
  end
end

vehicle = Vehicle.new("Toyota", "Camry")
puts vehicle.identify
puts "Class: #{vehicle.class}"
puts "Instance of Vehicle: #{vehicle.is_a?(Vehicle)}"
