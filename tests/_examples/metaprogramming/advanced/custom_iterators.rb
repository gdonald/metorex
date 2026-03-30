# Custom Iterators: Build iterator patterns using blocks
# Demonstrates: yield-like patterns, higher-order functions, closures

# Custom each_with_index using a counter
def each_with_index(array, &block)
  i = 0
  array.each do |item|
    block.call(item, i)
    i += 1
  end
end

puts "=== each_with_index ==="
each_with_index(["apple", "banana", "cherry"]) do |item, idx|
  puts "#{idx}: #{item}"
end

# Custom times iterator
def times(n, &block)
  i = 0
  while i < n
    block.call(i)
    i += 1
  end
end

puts ""
puts "=== times ==="
times(5) do |i|
  puts "Iteration #{i}"
end

# Custom reduce/fold
def reduce(array, initial, &block)
  acc = initial
  array.each do |item|
    acc = block.call(acc, item)
  end
  acc
end

puts ""
puts "=== reduce ==="
sum = reduce([1, 2, 3, 4, 5], 0) do |acc, n|
  acc + n
end
puts "Sum: #{sum}"

product = reduce([1, 2, 3, 4, 5], 1) do |acc, n|
  acc * n
end
puts "Product: #{product}"

# Custom flat_map
def flat_map(array, &block)
  result = []
  array.each do |item|
    mapped = block.call(item)
    mapped.each do |sub|
      result.push(sub)
    end
  end
  result
end

puts ""
puts "=== flat_map ==="
nested = [[1, 2], [3, 4], [5, 6]]
flattened = flat_map(nested) do |arr|
  arr
end
puts flattened

# Custom take_while
def take_while(array, &block)
  result = []
  done = false
  array.each do |item|
    if !done && block.call(item)
      result.push(item)
    else
      done = true
    end
  end
  result
end

puts ""
puts "=== take_while ==="
numbers = [2, 4, 6, 7, 8, 10]
evens = take_while(numbers) do |n|
  n % 2 == 0
end
puts evens
