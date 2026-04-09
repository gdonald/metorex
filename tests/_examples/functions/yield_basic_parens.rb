def greet(name)
  puts "before yield"
  yield name
  puts "after yield"
end

greet "Alice" do |n|
  puts "Hello, " + n
end

def repeat(n)
  i = 0
  while i < n
    yield i
    i = i + 1
  end
end

repeat 3 do |x|
  puts x
end

def with_return_value
  result = yield 10
  puts result
end

with_return_value do |x|
  x * 2
end

def no_args_yield
  yield
end

no_args_yield do
  puts "yielded with no args"
end
