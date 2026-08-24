class Tag
  def ==(other)
    other.kind_of? Tag
  end
end

class NeverEqual
  def ==(other)
    false
  end

  def equal?(other)
    false
  end
end

first = Tag.new
second = first.dup

puts(first == second)
puts(first === second)
puts(first === first)
puts(first === Object.new)

stubborn = NeverEqual.new
puts(stubborn == stubborn)
puts(stubborn === stubborn)
puts(stubborn === stubborn.dup)
