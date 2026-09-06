# A precision of zero or more leaves an Integer alone, since it has no digits
# to drop.
p(15.ceil)
p(15.ceil(0))
p(15.ceil(42))
p(15.floor(1))

# A negative precision rounds to that power of ten, each method sending a
# value between two multiples its own way.
p(15.ceil(-1))
p(15.floor(-1))
p(15.truncate(-1))
p(-15.truncate(-1))
p(-15.floor(-1))
p(249.round(-2))
p(250.round(-2))

# `half:` names where a value exactly between two multiples goes.
p(25.round(-1, half: :up))
p(25.round(-1, half: :down))
p(25.round(-1, half: :even))
p(35.round(-1, half: :even))
p(-25.round(-1, half: :up))
p(-25.round(-1, half: :down))

begin
  42.round(-1, half: :foo)
rescue ArgumentError => error
  puts(error.message)
end

# A sign glued to a numeric literal belongs to the literal.
p(-1.abs)
p(-2 ** 2)
p(+249.round(-2))
