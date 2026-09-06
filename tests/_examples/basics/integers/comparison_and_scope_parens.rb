# Comparable is mixed into the classes whose values have an order.
p(Integer.include?(Comparable))
p(Integer.ancestors)
p(String.include?(Comparable))

# A number compared against something that is not one answers by asking that
# object, and has no ordering against it at all.
class Measure
  def initialize(value)
    @value = value
  end

  def ==(other)
    other == @value
  end
end

p(5 == Measure.new(5))
p(5 == Measure.new(6))
p(5 <=> "five")

# `Foo::bar` names a method when the name is lowercase, and a constant when it
# is capitalized.
module Units
  METRIC = "metric"

  def self.name_of(value)
    "#{value} #{METRIC}"
  end
end

puts(Units::name_of(3))
puts(Units::METRIC)

# An Integer is its own numerator over a denominator of one.
p(34.numerator)
p(34.denominator)
p(34.to_r)
p(34.size)
