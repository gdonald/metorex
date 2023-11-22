class Animal
  def initialize(name)
    @name = name
  end

  def speak
    "Some sound"
  end

  def describe
    "I am an animal"
  end

  attr_reader :name
end

class Dog < Animal
  def initialize(name, breed)
    super(name)
    @breed = breed
  end

  def speak
    parent_sound = super()
    parent_sound + " -> Woof!"
  end

  def describe
    parent_desc = super()
    parent_desc + " named " + @name
  end

  attr_reader :breed
end

dog = Dog.new("Buddy", "Golden Retriever")
puts dog.name
puts dog.breed
puts dog.speak
puts dog.describe
