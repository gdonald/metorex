# A range over numbers counts its way along, and answers an arithmetic
# sequence when it is handed no block.
p((1..10).step(3).to_a)
p((1..10).step(3).class)
p((1.0...55.6).step(18.2).to_a)
p((1.0...55.6).step(18.2).size)

# A range over anything with a `succ` walks with it, handing out every step'th
# value it reaches.
p(("A".."G").step(2).to_a)
p(("A".."E").step.to_a)
p((:a..:e).step(2).to_a)

# A step of no width hands out the first value and no more, since the count it
# waits on never comes round again.
p(("A".."E").step(0).to_a)

# A step of another kind is added instead, and a range over anything but
# numbers answers a plain Enumerator whose size is unknown.
p(("A".."AAA").step("A").to_a)
p(("A".."E").step("A").class)
p(("A".."E").step("A").size)

# A step running against the range's direction reaches nothing.
p(("E".."A").step("A").to_a)

# A step that is no kind the beginning can be added to is refused.
begin
  ("A".."G").step([]) { }
rescue TypeError => error
  puts(error.message)
end

# A range with nothing at either end names no walk at all.
begin
  Range.new(nil, nil).step(1)
rescue ArgumentError => error
  puts(error.message)
end

begin
  Range.new(nil, nil).step
rescue ArgumentError => error
  puts(error.message)
end

# A step of some other kind over numbers is read through `coerce`, and only
# where the walk is about to run.
counter = Object.new
def counter.coerce(other)
  [other, 2]
end
p((1..5).step(counter).to_a)

plain = Object.new
p((1..2).step(plain).class)
begin
  (1..2).step(plain) { }
rescue TypeError => error
  puts(error.message)
end
