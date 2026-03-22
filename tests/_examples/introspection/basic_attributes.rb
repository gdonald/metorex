def greet(name)
  "Hello, " + name
end

def calculate(x, y)
  x + y
end

puts "greet.name = #{method(:greet).name}"
puts "calculate.name = #{method(:calculate).name}"
puts "greet.doc = #{nil}"
puts "calculate.doc = #{nil}"
