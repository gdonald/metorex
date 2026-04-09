a = [1, 2, 3, 4, 5]
puts(a.inject(0) { |sum, x| sum + x })
puts(a.inject(10) { |sum, x| sum + x })

b = a.dup
b << 6
puts(a.length)
puts(b.length)

puts([1, nil, 2, nil, 3].compact)
puts([1, [2, 3], [4, [5]]].flatten)
puts([3, 1, 2, 1, 3].uniq)
puts([5, 2, 8, 1].min)
puts([5, 2, 8, 1].max)
puts([].empty?)
puts([1].empty?)
puts([10, 20, 30].first)
puts([10, 20, 30].last)
puts([1, 2, 3].include?(2))
puts([1, 2, 3].include?(9))
