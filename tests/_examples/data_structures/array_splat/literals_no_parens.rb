# A splat inside an array literal splices its elements in place.

def collect(first, *rest)
  [first, *rest]
end

puts collect(:one).inspect
puts collect(:one, :two, :three).inspect

middle = [2, 3]
spliced = [1, *middle, 4]
puts spliced.inspect
copied = [*middle]
puts copied.inspect
from_nil = [*nil]
puts from_nil.inspect
with_string = [0, *"solo"]
puts with_string.inspect

nested = [[1, 2], [3]]
flat = [*nested]
puts flat.inspect
