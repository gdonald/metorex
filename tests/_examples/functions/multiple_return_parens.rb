def swap(a, b)
  return b, a
end

result = swap 1, 2
puts result[0]
puts result[1]

def triple
  return 10, 20, 30
end

r = triple()
puts r[0]
puts r[1]
puts r[2]
