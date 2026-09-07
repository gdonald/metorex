# Enumerable's predicates, batching, and counting, over a class that supplies
# only each.
class Numerous
  include Enumerable

  def initialize *list
    @list = list
  end

  def each
    @list.each { |value| yield value }
  end

  def size
    @list.size
  end
end

numbers = Numerous.new 1, 2, 3, 4, 5

p numbers.all? { |value| value > 0 }
p numbers.any? { |value| value > 4 }
p numbers.none? { |value| value > 5 }
p numbers.one? { |value| value == 3 }

# A pattern argument is matched with ===, and it stands in for a block.
p numbers.all? Integer
p numbers.any? String
p numbers.none? String
p numbers.count 3
p numbers.count { |value| value.odd? }
p numbers.find_index 4

# nil, true, and false answer to their own classes.
p [nil, false].any? NilClass
p [1, nil].none? TrueClass

p numbers.each_slice(2).to_a
p numbers.each_cons(2).to_a
p numbers.each_slice(2).size
p numbers.each_cons(2).size

p numbers.first
p numbers.first 2
p numbers.take_while { |value| value < 3 }
p numbers.drop 2
p numbers.drop_while { |value| value < 3 }

# A count is coerced through to_int, and a negative one is an error.
class Two
  def to_int
    2
  end
end
p numbers.take Two.new
begin
  numbers.take -1
rescue ArgumentError => error
  p error.message
end
begin
  numbers.drop "two"
rescue TypeError => error
  p error.message
end

p Numerous.new("a", "b", "a").tally
p Numerous.new("a", "b", "a").tally({ "a" => 1 })

p numbers.zip [10, 20, 30, 40, 50]
p numbers.flat_map { |value| [value, -value] }
p numbers.sum
p Numerous.new(2.78, 5.0, 2.5, 4.44, 3.89, 3.89, 4.44, 7.78, 5.0, 2.78, 5.0, 2.5).sum

# break in a block leaves the Enumerable method with the value it carried.
counted = Numerous.new(4, 3, 2, 1)
p(counted.take_while { |value| break :stopped if value == 3; true })

# An optional parameter takes its default even when a splat follows it.
def labelled name = :unnamed, *rest
  [name, rest]
end
p labelled()
p labelled :given, 1, 2
