# A Set keeps its elements in the order they were added, and holds any value
# with a stable rendering, not only numbers and strings.
numbers = Set.new [3, 1, 2, 1]
p numbers.to_a
p numbers.size
p numbers.include? 2
p numbers.inspect

mixed = Set.new [:a, "b", 3, [4, 5], nil]
p mixed.to_a
p mixed.include? [4, 5]

numbers << 4
p numbers.to_a
p numbers.add? 4
numbers.merge [5, 6]
p numbers.to_a
numbers.subtract [5, 6]
p numbers.to_a

# The algebra, spelled either as a method or as an operator.
left = Set.new [1, 2, 3]
right = Set.new [3, 4]

p (left | right).to_a
p (left + right).to_a
p (left - right).to_a
p (left & right).to_a
p (left ^ right).to_a
p left.union(right).to_a
p left.difference([3]).to_a
p left.intersection([2, 3, 9]).to_a

# The comparisons, which take a Set on either spelling.
p left.subset? Set.new([1, 2, 3, 4])
p left.superset? Set.new([1, 2])
p left.proper_subset? Set.new([1, 2, 3])
p(left <= Set.new([1, 2, 3]))
p(left < Set.new([1, 2, 3, 4]))
p left.disjoint? Set.new([9])
p left.intersect? Set.new([3])
p(Set.new([1, 2]) == Set.new([2, 1]))

# The walk, which Enumerable builds on.
p left.map { |value| value * 2 }
p left.select { |value| value.odd? }
p left.sort
p left.each.to_a
p left.join "-"

# The in-place filters answer the set itself.
kept = Set.new [1, 2, 3, 4]
kept.keep_if { |value| value.even? }
p kept.to_a
dropped = Set.new [1, 2, 3, 4]
dropped.delete_if { |value| value.even? }
p dropped.to_a
mapped = Set.new [1, 2, 3]
mapped.map! { |value| value + 10 }
p mapped.to_a

replaced = Set.new [1, 2]
replaced.replace [7, 8]
p replaced.to_a
replaced.clear
p replaced.to_a
p replaced.empty?
