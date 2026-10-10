# Calls of a block through yield and through a Proc.
def each_square(limit)
  index = 0
  while index < limit
    yield index * index
    index += 1
  end
end

total = 0
each_square(200_000) { |square| total += square }
adder = proc { |value| total += value }
100_000.times { |step| adder.call(step) }
puts total
