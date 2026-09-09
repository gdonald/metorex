# A walk over evenly spaced numbers, and the Cartesian product of walks.

sequence = 1.step(10, 3)
p sequence.class
p sequence.to_a
p [sequence.begin, sequence.end, sequence.step, sequence.exclude_end?]
p sequence.inspect
p sequence.size

seen = []
1.step(10, 4) { |value| seen.push(value) }
p seen

p (1..10).step(3).to_a
p (1...10).step(4).last
p (1...10).step.size
p (1..10).step(3).inspect
p (1...10).step.exclude_end?
p 1.step(Float::INFINITY).size

p ((1..10) % 2).to_a
p ((1..10) % 2).inspect
p (1..10).step(100) == 1.step(10, 100)

product = Enumerator::Product.new([1, 2], [:a, :b])
p product.to_a
p product.size
p product.inspect
p product.each.size

pairs = []
product.each { |left, right| pairs.push([right, left]) }
p pairs

# A Yielder hands everything it is given to the block it was made with.
collected = []
yielder = Enumerator::Yielder.new { |*values| collected.push(values); "done" }
p yielder.yield(1, 2)
yielder << 3
p collected
p yielder.to_proc.class

counted = Enumerator.new { |y| y << 1; y.yield 2, 3; y << [4] }
p counted.to_a

begin
  Enumerator::ArithmeticSequence.new
rescue NoMethodError => error
  p error.class
end

begin
  Object.new.each_entry { }
rescue NoMethodError => error
  p error.message
end
