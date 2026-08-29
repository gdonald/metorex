class Ledger
  attr_accessor :entries, :label

  def initialize
    @entries = []
    @label = "original"
  end

  def initialize_copy(source)
    @entries = source.entries.dup
    @label = "copy of #{source.label}"
  end
end

original = Ledger.new
original.entries << "first"

copy = original.dup
copy.entries << "second"

puts original.label
puts copy.label
puts original.entries.length
puts copy.entries.length
puts copy.equal?(original)

class Marker
  def note
    object_id
  end
end

marker = Marker.new
puts marker.note == marker.object_id

rational = Rational(1, 3)
puts rational.dup.equal?(rational)

complex_value = Complex(1.3, 3.1)
puts complex_value.dup.equal?(complex_value)
