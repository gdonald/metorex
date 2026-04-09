# inject with initial value
a = [1, 2, 3, 4, 5]
puts(a.inject(0) { |sum, x| sum + x })
puts(a.inject(10) { |sum, x| sum + x })
puts(a.inject("") { |s, x| s + x.to_s })

# dup / clone
b = [10, 20, 30]
c = b.dup
c << 40
puts(b.length)
puts(c.length)
d = b.clone
puts(d.length)

# flatten
puts([1, [2, 3], [4]].flatten)

# compact
puts([1, nil, 2, nil, 3].compact)

# empty?
puts([].empty?)
puts([1].empty?)

# first / last
puts([10, 20, 30].first)
puts([10, 20, 30].last)
puts([].first)
puts([].last)

# include? / contains?
puts([1, 2, 3].include?(2))
puts([1, 2, 3].include?(9))
puts([1, 2, 3].contains?(1))

# min / max
puts([5, 2, 8, 1].min)
puts([5, 2, 8, 1].max)
puts([3.5, 1.2, 2.8].min)
puts([3.5, 1.2, 2.8].max)
puts([].min)
puts([].max)

# uniq
puts([3, 1, 2, 1, 3].uniq)
