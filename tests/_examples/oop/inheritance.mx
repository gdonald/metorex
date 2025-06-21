class Animal
  def initialize(name)
    @name = name
  end

  def speak
    "Some sound"
  end

  def introduce
    @name + " says " + speak
  end
end

class Dog < Animal
  def speak
    "Woof!"
  end
end

class Cat < Animal
  def speak
    "Meow!"
  end
end

dog = Dog.new("Buddy")
cat = Cat.new("Whiskers")

puts dog.introduce
puts cat.introduce
