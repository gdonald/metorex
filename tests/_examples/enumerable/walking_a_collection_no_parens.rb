# Enumerable written over each: a class supplies each, and the module supplies
# the rest of the walk.
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

class Counted < Numerous
  attr_reader :arguments_passed

  def initialize *list
    super(*list)
  end

  def each *arguments
    @arguments_passed = arguments
    @list.each { |value| yield value }
  end
end

numbers = Numerous.new 4, 1, 3, 2

p numbers.to_a
p numbers.map { |value| value * 2 }
p numbers.select { |value| value.odd? }
p numbers.reject { |value| value.odd? }
p numbers.partition { |value| value > 2 }
p numbers.group_by { |value| value.odd? }
p numbers.sort
p numbers.sort_by { |value| -value }
p numbers.min
p numbers.max
p numbers.min 2
p numbers.max 2
p numbers.min_by { |value| -value }
p numbers.max_by { |value| -value }
p numbers.minmax
p numbers.include? 3
p numbers.take 2
p numbers.first
p numbers.each_with_index.to_a
p numbers.reverse_each.to_a
p numbers.inject :+

# An Enumerator handed back for a missing block reports the size of what it
# would walk, without walking it.
p numbers.map.size
p numbers.select.size

# find takes an ifnone callable, used only when nothing matches.
absent = lambda { "none" }
p numbers.find(absent) { |value| value > 10 }
p numbers.find(absent) { |value| value > 3 }

# each that yields several values at once hands the block one packed array.
class Pairs
  include Enumerable

  def each
    yield 1, 2
    yield 3, 4
  end
end

p Pairs.new.to_a
p Pairs.new.map { |pair| pair }
p Pairs.new.select { |pair| pair == [3, 4] }

# A splat forwards every element through super.
counted = Counted.new 5, 6, 7
p counted.to_a(:hello, "world")
p counted.arguments_passed

# return inside a block written in an Enumerable method leaves that method,
# not the each it was handed to.
class Finder
  include Enumerable

  def each
    yield 1
    yield 2
    yield 3
  end

  def first_even
    each do |value|
      return value if value.even?
    end
    nil
  end
end

p Finder.new.first_even
