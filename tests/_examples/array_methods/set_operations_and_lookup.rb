letters = [:a, :b, :b, :c]
p letters.union([:c, :d])
p letters.intersection([:b, :c, :e])
p letters.difference([:b])
p letters.union
p([1, 2] + [3])
p([1, 2, 2, 3] - [2])
p([1, 2] | [2, 3])
p([1, 2, 3] & [2, 3, 4])
p([1, 2] * 3)
p([1, 2] * ", ")

p([1, 2] <=> [1, 3])
p([1, 2] <=> [1, 2, 3])
p([1, 2] <=> [1, "two"])

pairs = [[:one, 1], [:two, 2]]
p pairs.assoc(:two)
p pairs.rassoc(1)
p pairs.assoc(:missing)

numbers = [10, 20, 30, 40]
p numbers.fetch(1)
p numbers.fetch(-1)
p numbers.fetch(9, :none)
p numbers.fetch(9) { |index| index * 2 }
p numbers.fetch_values(0, 2)
p numbers.values_at(0, 2..3)
p numbers.values_at(1..)
p numbers.values_at(..1)
p numbers.values_at(9)

begin
  numbers.fetch(9)
rescue IndexError => error
  puts error.message
end

p numbers.first(2)
p numbers.last(2)
p numbers.min
p numbers.max
p numbers.minmax
p numbers.max { |left, right| right <=> left }

stack = [1, 2, 3, 4]
p stack.pop(2)
p stack.shift(1)
p stack
p stack.insert(0, :front)
p stack.insert(-1, :back)

grid = [[1, 2], [3, 4]]
p grid.transpose
begin
  [[1, 2], [3]].transpose
rescue IndexError => error
  puts error.message
end

p [1, 2].zip([3, 4], [5, 6])
p [1, 2, 3].replace([:x])

begin
  [].first(2**70)
rescue RangeError => error
  puts error.class
end
