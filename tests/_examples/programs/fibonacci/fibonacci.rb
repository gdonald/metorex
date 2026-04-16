# Fibonacci sequence using both iterative and recursive approaches

# Iterative Fibonacci
def fibonacci_iterative(n)
  if n <= 1
    return n
  end

  a = 0
  b = 1
  for i in 2..n
    temp = b
    b = a + b
    a = temp
  end
  b
end

# Recursive Fibonacci
def fibonacci_recursive(n)
  if n <= 1
    return n
  end
  fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2)
end

# Print first 10 Fibonacci numbers (iterative)
puts "Iterative:"
for i in 0..9
  puts fibonacci_iterative(i)
end

# Print first 10 Fibonacci numbers (recursive)
puts "Recursive:"
for i in 0..9
  puts fibonacci_recursive(i)
end
