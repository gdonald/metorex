# Sorting asks for a number of comparisons that grows as n log n, so a
# thousand elements take about ten thousand rather than half a million.
numbers = (1..1000).to_a.shuffle(random: Random.new(7))

comparisons = 0
sorted = numbers.sort do |left, right|
  comparisons += 1
  left <=> right
end
p(sorted == (1..1000).to_a)
p(comparisons < 20_000)

keys = 0
by_key = numbers.sort_by { |number| keys += 1; -number }
p(by_key.first(3))
p(keys)

words = %w[pear fig apple kiwi date]
p(words.sort)
p(words.sort { |left, right| right <=> left })
p(words.sort_by(&:size))
p([3, 1, 2].sort!)
