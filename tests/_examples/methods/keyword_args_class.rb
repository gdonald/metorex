class Person
  def initialize(name:, age: 0)
    @name = name
    @age = age
  end

  def introduce
    puts "I'm #{@name}, age #{@age}"
  end
end

p1 = Person.new(name: "Alice")
p1.introduce

p2 = Person.new(name: "Bob", age: 30)
p2.introduce
