numbers = [3, 1, nil, 2]
p numbers.compact!
p numbers
p numbers.compact!
p numbers.sort!
p numbers.reverse!
p numbers.map! { |value| value * 2 }
p numbers.reject! { |value| value > 4 }
p numbers.select! { |value| value > 0 }
p numbers.uniq!
p [1, 1, 2].uniq { |value| value }

p [1, 2, 3].rotate
p([1, 2, 3].rotate 2)
p [1, [2, [3, [4]]]].flatten
p([1, [2, [3, [4]]]].flatten 1)

p([10, 20, 30].at 1)
p([10, 20, 30].at -1)
p([10, 20, 30].at 9)
p [:a, :b, :b].count
p([:a, :b, :b].count :b)
p [1, 2, 3].count { |value| value.odd? }
p [1, 2, 3, 1].take_while { |value| value < 3 }
p [1, 2, 3, 1].drop_while { |value| value < 3 }
p ["333", "2", "60"].minmax
p [6, 4, 10].min
p [6, 4, 10].max { |left, right| right <=> left }

p [1, 2, 3].each.class.to_s
p [1, 2, 3].map.class.to_s
p([1].eql? [1.0])
p([1].eql? [1])

class Stack < Array
  def initialize(first, second)
    self << first << second
  end
end

stack = Stack.new 1, 2
p stack
p stack.class
p stack.size
p stack == [1, 2]
stack << 3
p stack.to_a
p Stack[7, 8, 9].to_a

cycle = []
cycle << cycle
p cycle
p cycle == cycle

grown = [1, 2, 3]
seen = []
grown.each do |value|
  seen << value
  grown << 4 if value == 3 && grown.size == 3
end
p seen
