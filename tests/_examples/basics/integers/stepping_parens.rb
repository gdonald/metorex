# A step named by keyword walks the same ground as one given by position.
p(1.step(9, 4).to_a)
p(1.step(to: 9, by: 4).to_a)
p(1.step(9, by: 4).to_a)
p(1.step(by: 4).first(3))

# A walk with no end has no count, while an infinite step reaches its end in
# a single stride.
p(1.step(by: 42).size)
p(1.step(to: Float::INFINITY, by: 42).size)
p(1.step(to: Float::INFINITY, by: Float::INFINITY).size)
p(1.step(to: -Float::INFINITY, by: -Float::INFINITY).size)

# An end or a step given both by position and by keyword is refused, as is a
# keyword the method does not take.
begin
  1.step(5, 1, to: 5) { break }
rescue ArgumentError => error
  puts(error.message)
end

begin
  1.step(5, 1, by: 5) { break }
rescue ArgumentError => error
  puts(error.message)
end

begin
  1.step(step: 2) { break }
rescue ArgumentError => error
  puts(error.message)
end

# A step that cannot be compared against zero is refused when the walk runs
# rather than when it is built, so asking for the walk alone hands back an
# ordinary Enumerator.
begin
  1.1.step(5.1, "foo") { }
rescue ArgumentError => error
  puts(error.message)
end

walk = 1.step(5, "foo")
p(walk.class)
begin
  walk.size
rescue ArgumentError => error
  puts(error.message)
end

# A walk from an infinity to the same infinity has a span that is no number,
# so it yields nothing and its count is refused.
collected = []
Float::INFINITY.step(Float::INFINITY, 1) { |value| collected.push(value) }
p(collected)

begin
  Float::INFINITY.step(Float::INFINITY, 1).size
rescue FloatDomainError => error
  puts(error.message)
end
