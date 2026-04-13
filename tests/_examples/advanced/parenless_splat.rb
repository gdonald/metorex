# Test paren-less call with splat as non-first arg
def collect(a, *rest)
  [a] + rest
end
arr = [2, 3, 4]
result = collect 1, *arr
puts result.length
puts result[0]
puts result[1]
