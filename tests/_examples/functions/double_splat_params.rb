def collect(a, **rest)
  [a, rest.size, rest[:b], rest[:c]]
end

puts collect(1).inspect
puts collect(1, b: 2, c: 3).inspect

def mixed(a, b: 5, **rest)
  [a, b, rest.size, rest[:c]]
end

puts mixed(1, b: 2, c: 3).inspect
puts mixed(1, c: 3).inspect

def anonymous(a, **)
  a
end

puts anonymous(1, b: 2).inspect

def parenless a, **rest
  [a, rest.size]
end

puts parenless(1, b: 2).inspect
