def greet(name)
  "Hello, " + name
end

def calculate(x, y)
  x + y
end

greet_location = method(:greet).source_location
calculate_location = method(:calculate).source_location

puts "greet defined in #{greet_location[0].split('/').last} line #{greet_location[1]}"
puts "calculate defined in #{calculate_location[0].split('/').last} line #{calculate_location[1]}"
