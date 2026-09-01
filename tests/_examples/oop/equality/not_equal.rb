# `!=` is the negation of `==`, so defining only `==` gives a class both.

class Tagged
  attr_reader :tag

  def initialize(tag)
    @tag = tag
  end

  def ==(other)
    other.is_a?(Tagged) && tag == other.tag
  end
end

first = Tagged.new(:a)
same = Tagged.new(:a)
other = Tagged.new(:b)

puts (first == same).to_s
puts (first != same).to_s
puts (first != other).to_s
puts first.send(:!=, same).to_s
puts first.send(:!=, other).to_s

# A class whose `==` always answers true is never unequal.
class AlwaysEqual
  def ==(other)
    true
  end
end

always = AlwaysEqual.new
puts (always != 1).to_s
puts (always != "anything").to_s

# Without a definition, `!=` falls back to identity for plain objects.
plain = Object.new
puts (plain != plain).to_s
puts (plain != Object.new).to_s
