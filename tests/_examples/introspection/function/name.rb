def greet(name)
  "Hello, " + name
end

def calculate(x, y)
  x + y
end

puts method(:greet).name
puts method(:calculate).name
