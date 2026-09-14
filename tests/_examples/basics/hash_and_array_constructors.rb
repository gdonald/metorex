# `Hash[]` reads a list of pairs, a hash, or one argument that answers
# `to_hash` or `to_ary`, and refuses an element that names no pair.
p(Hash[[[:a, :b], [:c, :d]]])
p(Hash[[[:a]]])
begin
  Hash[[:a]]
rescue ArgumentError => refused
  p(refused.message)
end
begin
  Hash[[[:a, :b, :c]]]
rescue ArgumentError => refused
  p(refused.message)
end

class Pairs
  def to_ary
    [[:a, :b]]
  end
end
p(Hash[Pairs.new])

# `Array.new` given an array takes its elements rather than a size, and
# refuses a default alongside one.
p(Array.new([4, 5, 6]))

class Listed
  def to_ary
    [1, 2]
  end
end
p(Array.new(Listed.new))
begin
  Array.new([1, 2], 1)
rescue TypeError => refused
  p(refused.class)
end
