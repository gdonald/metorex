class Version
  include Comparable

  def initialize value
    @value = value
  end

  def to_i
    @value
  end

  def <=> other
    to_i <=> other.to_i
  end
end

low = Version.new(100)
high = Version.new(300)
inside = Version.new(200)
above = Version.new(400)
below = Version.new(50)

exclusive = low...high
inclusive = low..high

puts exclusive.include?(inside).inspect
puts exclusive.include?(above).inspect
puts exclusive.include?(below).inspect
puts exclusive.include?(low).inspect
puts exclusive.include?(high).inspect
puts inclusive.include?(high).inspect
numbers = 1...5
puts numbers.include?(3).inspect
letters = "a".."c"
puts letters.include?("b").inspect
