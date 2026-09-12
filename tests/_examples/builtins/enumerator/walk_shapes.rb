# A walk names the count it will hand out, reports itself before it has been
# given anything to walk, and a chain rewinds what it has run.
sized = Enumerator.new(100) { |yielder| yielder << 1 }
p sized.size

base = 100
counted = Enumerator.new(-> { base + 1 }) { |yielder| yielder << 1 }
base = 200
p counted.size

p Enumerator.allocate.inspect
p (1..3).each.inspect
p (1..3).each_slice(2).inspect
p (1..3).each.size

generator = Enumerator::Generator.new do |yielder, *extra|
  yielder << 3 << 2
  yielder << extra unless extra.empty?
  :answered
end
p generator.is_a?(Enumerable)
collected = []
p generator.each(:more) { |value| collected << value }
p collected

chain = Enumerator::Chain.new 1..2, 3..4
p chain.to_a
p chain.rewind.equal?(chain)

p [1, 2, 3, 4].to_enum.each_with_index { |value, index| value }
