class Greeter
  def greet(name, greeting = "Hello")
    puts greeting + ", " + name
  end
end

g = Greeter.new
g.greet("Alice")
g.greet("Bob", "Hi")
