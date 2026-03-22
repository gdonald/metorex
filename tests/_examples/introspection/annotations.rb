def add(x, y)  # int, int -> int
  x + y
end

def greet(name)  # str -> str
  "Hello, " + name
end

def process(data, count, flag)  # str, int, bool
  data
end

puts "add.parameters = #{method(:add).parameters}"
puts "greet.parameters = #{method(:greet).parameters}"
puts "process.parameters = #{method(:process).parameters}"

def no_annotations(a, b)
  a + b
end

puts "no_annotations.parameters = #{method(:no_annotations).parameters}"
