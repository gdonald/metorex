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

  def get_name
    @name
  end
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

  def get_breed
    @breed
  end
end

dog = Dog.new("Buddy", "Golden Retriever")
puts dog.get_name
puts dog.get_breed
puts dog.speak
puts dog.describe
