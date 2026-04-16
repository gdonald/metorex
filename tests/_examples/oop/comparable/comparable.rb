# Comparable module with spaceship operator
class Weight
  include Comparable

  attr_accessor :value

  def initialize(v)
    @value = v
  end

  def <=>(other)
    @value <=> other.value
  end
end

a = Weight.new 10
b = Weight.new 20

puts a < b
puts a > b
puts a <= b
puts a >= b
puts b > a
puts b >= a
puts a < a
puts a <= a
