# A copy of a hash with String keys keeps its own keys: adding to the copy
# or to the original leaves the other as it was.
original = { "alpha" => 1 }
copied = original.dup
copied["beta"] = 2
original["gamma"] = 3
cloned = original.clone
cloned["delta"] = 4
p original.keys
p copied.keys
p cloned.keys
p original.keys.map(&:frozen?)
many = {}
1_000.times { |step| many["key#{step}"] = step }
p [many.size, many["key999"], many.keys.last.frozen?]
