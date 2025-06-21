def simple_func
  1
end

puts method(:simple_func).name
puts method(:simple_func).name
puts nil

def outer(x)
  y = 10

  inner = lambda do |z|
    x + y + z
  end

  puts inner.class.name
  puts inner.class.name
  puts inner.binding

  inner.call(5)
end

result = outer(3)
puts result
