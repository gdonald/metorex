# A range written with literal ends is one object, however many times the line
# it sits on is run.
built = []
2.times { built.push(1..3) }
p built[0].equal?(built[1])

# Two ranges written in different places are two objects, even over the same
# values.
p((1..3).equal?(1..3))
p((1..3) == (1..3))

# A range a program writes is frozen, `dup` answers one that is not, and
# `clone` keeps the frozen state.
p((1..2).frozen?)
p((1..2).dup.frozen?)
p((1..2).clone.frozen?)

original = ("a"..."z")
copy = original.dup
p copy.begin
p copy.end
p copy.exclude_end?
p copy.equal?(original)
