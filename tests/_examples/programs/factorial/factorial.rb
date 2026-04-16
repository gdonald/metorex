# Factorial: iterative and recursive implementations

# Iterative factorial
def factorial_iterative(n)
  if n <= 1
    return 1
  end
  result = 1
  for i in 2..n
    result *= i
  end
  result
end

# Recursive factorial
def factorial_recursive(n)
  if n <= 1
    return 1
  end
  n * factorial_recursive(n - 1)
end

# Test both implementations
for n in 0..10
  iter = factorial_iterative(n)
  recur = factorial_recursive(n)
  puts "#{n}! = #{iter}"
end
