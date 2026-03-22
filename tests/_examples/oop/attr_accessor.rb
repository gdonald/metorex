class Person
  attr_accessor :name, :age, :email

  def initialize(name, age, email)
    @name = name
    @age = age
    @email = email
  end
end

p = Person.new("Charlie", 35, "charlie@example.com")
puts p.name
puts p.age
puts p.email
p.name = "Charles"
p.age = 36
p.email = "charles@example.com"
puts p.name
puts p.age
puts p.email
