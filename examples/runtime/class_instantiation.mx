# Class and Instance Creation Example
# Demonstrates defining classes, creating instances, and calling initialize constructors

# Simple class without constructor
class Point
  def initialize(x, y)
    @x = x
    @y = y
  end

  def get_x()
    @x
  end

  def get_y()
    @y
  end

  def to_s()
    "Point(#{@x}, #{@y})"
  end
end

# Create instances
p1 = Point(10, 20)
p2 = Point(5, 15)

puts "p1 = #{p1.to_s()}"
puts "p2 = #{p2.to_s()}"
puts "p1.x = #{p1.get_x()}"
puts "p1.y = #{p1.get_y()}"

# Class with instance variables and class variables
class Counter
  @@count = 0

  def initialize(name)
    @name = name
    @@count = @@count + 1
    @id = @@count
  end

  def get_name()
    @name
  end

  def get_id()
    @id
  end

  def get_total_count()
    @@count
  end
end

c1 = Counter("First")
c2 = Counter("Second")
c3 = Counter("Third")

puts "Counter #{c1.get_name()}: ID = #{c1.get_id()}"
puts "Counter #{c2.get_name()}: ID = #{c2.get_id()}"
puts "Counter #{c3.get_name()}: ID = #{c3.get_id()}"
puts "Total counters created: #{c1.get_total_count()}"

# Class with inheritance
class Animal
  def initialize(name)
    @name = name
  end

  def get_name()
    @name
  end

  def speak()
    "Some sound"
  end
end

class Dog < Animal
  def initialize(name, breed)
    @name = name
    @breed = breed
  end

  def get_breed()
    @breed
  end

  def speak()
    "Woof!"
  end
end

dog = Dog("Buddy", "Golden Retriever")
puts "Dog name: #{dog.get_name()}"
puts "Dog breed: #{dog.get_breed()}"
puts "Dog says: #{dog.speak()}"

# Class without initialize method
class Simple
  def greet()
    "Hello from Simple class"
  end
end

s = Simple()
puts s.greet()
