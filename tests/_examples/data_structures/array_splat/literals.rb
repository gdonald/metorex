# A splat inside an array literal splices its elements in place.

def collect(first, *rest)
  [first, *rest]
end

puts collect(:one).inspect
puts collect(:one, :two, :three).inspect

middle = [2, 3]
puts([1, *middle, 4].inspect)
puts([*middle].inspect)
puts([*nil].inspect)
puts([0, *"solo"].inspect)

nested = [[1, 2], [3]]
puts([*nested].inspect)
