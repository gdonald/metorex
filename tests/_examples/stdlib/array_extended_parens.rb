a = [1, 2, 3, 4, 5]
puts a.inject(0) { |sum, x| sum + x }
puts a.inject(10) { |sum, x| sum + x }

b = a.dup
b << 6
puts a.length
puts b.length

c = [1, nil, 2, nil, 3]
puts c.compact

d = [1, [2, 3], [4, [5]]]
puts d.flatten

e = [3, 1, 2, 1, 3]
puts e.uniq

f = [5, 2, 8, 1]
puts f.min
puts f.max

empty = []
puts empty.empty?
g = [1]
puts g.empty?
h = [10, 20, 30]
puts h.first
puts h.last
puts h.include?(20)
puts h.include?(99)
