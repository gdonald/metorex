# Integer arithmetic past the word width stays exact
big = 4611686018427387903
puts big + big
puts big * 3
puts 0 - big - big - 2

# String comparisons
puts "abc" < "abd"
puts "abd" > "abc"
puts "abc" <= "abc"
puts "abc" >= "abc"
puts "abc" <= "abd"
puts "abd" >= "abc"

# Bitwise on integers
puts 5 & 3
puts 5 | 3
puts 5 ^ 3

# Custom == via method
class Eq
  def initialize(val)
    @val = val
  end

  def ==(other)
    @val == other.instance_variable_get(:@val)
  end
end

puts Eq.new(1) == Eq.new(1)
puts Eq.new(1) == Eq.new(2)

# Custom <=> via method
class Cmp
  include Comparable

  def initialize(val)
    @val = val
  end

  def <=>(other)
    @val <=> other.instance_variable_get(:@val)
  end
end

puts Cmp.new(1) < Cmp.new(2)
puts Cmp.new(2) > Cmp.new(1)
puts Cmp.new(1) <= Cmp.new(1)
puts Cmp.new(1) >= Cmp.new(1)
