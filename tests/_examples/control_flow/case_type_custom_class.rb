# Type pattern matching with custom classes

class Dog
  def initialize(name)
    @name = name
  end

  def bark
    puts "#{@name} says woof!"
  end
end

class Cat
  def initialize(name)
    @name = name
  end

  def meow
    puts "#{@name} says meow!"
  end
end

# Test with Dog instance
animal = Dog.new("Buddy")
case animal
when Dog
  puts "It's a dog!"
  animal.bark
when Cat
  puts "It's a cat!"
  animal.meow
else
  puts "Unknown animal"
end

# Test with Cat instance
animal = Cat.new("Whiskers")
case animal
when Dog
  puts "It's a dog!"
  animal.bark
when Cat
  puts "It's a cat!"
  animal.meow
else
  puts "Unknown animal"
end

# Test with non-class value
animal = "not an animal"
case animal
when Dog
  puts "It's a dog!"
when Cat
  puts "It's a cat!"
when String
  puts "It's just a string"
else
  puts "Unknown type"
end
