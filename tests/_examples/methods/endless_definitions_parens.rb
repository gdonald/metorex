def doubled(value) = value * 2
puts(doubled(21))

class Counter
  def self.zero = 0
  def value = 42
  def sum(a, b) = a + b
  def uses_other = value + 1
end

puts(Counter.zero)
puts(Counter.new.value)
puts(Counter.new.sum(1, 2))
puts(Counter.new.uses_other)

module Named
  def self.label = "named"
end

puts(Named.label)
