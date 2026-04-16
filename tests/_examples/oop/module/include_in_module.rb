module Greetable
  def greet
    "hello"
  end
end

class Person
  include Greetable
end

puts Person.new.greet
