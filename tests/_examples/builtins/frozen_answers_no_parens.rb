# What the runtime hands back over and over is frozen, so changing it cannot
# reach what everyone else is given.
p [nil.to_s.frozen?, true.to_s.frozen?, false.to_s.frozen?]
p Comparable.name.frozen?
found = /(.)(.)/.match "ab"
p found.string.frozen?

# A range holds its ends and nothing else, and every one is frozen. An
# instance of a subclass is an ordinary object, so it is not.
p [(1..2).frozen?, Range.new(1, 2).frozen?]
p Class.new(Range).new(1, 2).frozen?

# A subclass instance still stands wherever a range does.
walk = Class.new(Range).new 1, 4
p walk.to_a
same = (1..4) == walk
p same

# `clone` carries the frozen state across and `dup` leaves it behind, and
# `freeze:` names the copy's state outright.
held = [1, 2].freeze
p [held.clone.frozen?, held.dup.frozen?, held.clone(freeze: false).frozen?]
