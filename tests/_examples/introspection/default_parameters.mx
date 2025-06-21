def no_defaults(a, b)
  a + b
end

def with_defaults(a, b=10, c=20)
  a + b + c
end

def all_defaults(x=1, y=2, z=3)
  x + y + z
end

def greet(name, greeting="Hello", punctuation="!")
  greeting + " " + name + punctuation
end

puts method(:no_defaults).name
puts method(:no_defaults).parameters

puts method(:with_defaults).name
puts method(:with_defaults).parameters

puts method(:all_defaults).name
puts method(:all_defaults).parameters

puts method(:greet).name
puts method(:greet).parameters
