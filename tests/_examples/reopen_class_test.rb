module Greetable
  def greet
    "hello"
  end
end

class MyClass
end

class MyClass
  include Greetable
end

obj = MyClass.new
puts obj.greet
