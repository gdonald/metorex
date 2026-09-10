# A seeded generator answers the same numbers Ruby's own does, since both
# draw from the Mersenne Twister.
p Random.new(33).bytes 2
p Random.new(2 ** (63 * 4)).bytes 2
p Random.new(1234).rand
p Random.new(42).rand(0.0...100.0)
p Random.new(42) == Random.new(42)
p Random.new(42).seed
p Random.urandom(8).bytesize
p Random.new_seed.is_a? Integer

# A Range draws from the width between its ends, so an end that subtracts and
# adds bounds a draw of its own kind.
Kernel.srand 176542
p Kernel.rand(3..5).between?(3, 5)

Counter = Struct.new :value do
  def to_int
    value
  end

  def <=> other
    to_int <=> other.to_int
  end

  def - other
    self.class.new to_int - other.to_int
  end

  def + other
    self.class.new to_int + other.to_int
  end
end
p rand(Counter.new(1)..Counter.new(42)).is_a? Counter
begin
  rand Object.new..67
rescue ArgumentError => problem
  p problem.message
end
