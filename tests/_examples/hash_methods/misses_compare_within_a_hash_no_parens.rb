# A key a hash does not hold is compared with `eql?` against the keys that
# share its `#hash`, not against every key. Array keys are placed by their
# `#hash`, which reads their elements' `#hash`.
class Tagged
  @@compared = 0

  def self.compared
    @@compared
  end

  attr_reader :id

  def initialize id
    @id = id
  end

  def hash
    @id % 4
  end

  def eql?(other)
    @@compared += 1
    other.is_a?(Tagged) && other.id == id
  end
end

held = {}
400.times { |step| held[[Tagged.new(step)]] = step }

before = Tagged.compared
p held[[Tagged.new(-1)]]
missed = Tagged.compared - before
p missed < held.size / 2

p held[[Tagged.new(7)]]
p held.key?([Tagged.new(399)])
p held.size
p held.shift.last
copy = held.dup
held.delete([Tagged.new(1)])
p [held.size, copy.size, copy[[Tagged.new(1)]]]
