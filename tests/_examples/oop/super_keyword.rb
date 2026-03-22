class Animal
  def initialize(name)
    @name = name
  end

  def speak
    puts "#{@name} makes a sound"
  end

  def describe
    "Animal: #{@name}"
  end
end

class Dog < Animal
  def initialize(name, breed)
    super(name)
    @breed = breed
  end

  def speak
    super
    puts "#{@name} barks"
  end

  def describe
    super + ", Breed: #{@breed}"
  end
end

d = Dog.new("Rex", "Labrador")
d.speak
puts d.describe
