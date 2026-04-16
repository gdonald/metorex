# Set methods demonstration

# Creating sets
s = Set.new
s.add(1)
s.add(2)
s.add(3)
s.add(2)
puts s.size

# From array
s2 = Set.new([10, 20, 30, 20])
puts s2.size

# Membership
puts s.contains?(2)
puts s.contains?(99)

# Remove
s.remove(2)
puts s.size

# empty?
puts Set.new.empty?
puts s.empty?

# Set operations
a = Set.new([1, 2, 3, 4])
b = Set.new([3, 4, 5, 6])

puts a.union(b).size
puts a.intersection(b).size
puts a.difference(b).size

# each
total = 0
Set.new([10, 20, 30]).each { |x| total += 1 }
puts total

# to_a
puts Set.new([1, 2, 3]).to_a.length
