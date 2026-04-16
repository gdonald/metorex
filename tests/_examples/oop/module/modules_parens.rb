module Greetable
  def greet
    puts("Hello, I am #{@name}")
  end

  def farewell
    puts("Goodbye from #{@name}")
  end
end

module ClassMethods
  def description
    puts("I am a class with module methods")
  end
end

class Person
  include Greetable
  extend ClassMethods

  def initialize(name)
    @name = name
  end
end

p = Person.new("Alice")
p.greet
p.farewell
Person.description
