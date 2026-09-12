# A collection is held by reference, so each one carries methods of its own.
held = [1, 2]
def held.doubled
  map { |number| number * 2 }
end
p(held.doubled)
p(held.respond_to?(:doubled))
p([1, 2].respond_to?(:doubled))

# `clone` carries those methods over where `dup` leaves them behind.
p(held.clone.respond_to?(:doubled))
p(held.dup.respond_to?(:doubled))

# A subclass of a collection answers its own methods rather than the ones
# the collection would answer.
class Bag < Hash
  def size
    77
  end
end
bag = Bag.new
bag["a"] = "b"
p(bag.size)
p(bag.length)
