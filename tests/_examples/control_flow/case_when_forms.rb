# A `when` with no subject tests each value for truth, and a splat there names
# each of the values it holds as a choice of its own.
answered = case
when *[false]
  "first"
when *[true]
  "second"
end
p(answered)

# A `when` clause holds an expression, never a name to bind, so a name written
# there stands for what it holds.
small = ->(number) { number < 10 }
even = ->(number) { number.even? }
def describe(value, small, even)
  case value
  when even then "even"
  when small then "small and odd"
  else "large and odd"
  end
end
p(describe(4, small, even))
p(describe(7, small, even))
p(describe(11, small, even))

# `next` hands a value back from a block, and splatting nil hands back nothing.
def taking
  yield
end
p(taking { next *nil })
p(taking { next *[1, 2] })
p(taking { next 42 })

held = [1, 2, 3].map { |number| next number * 2 if number.odd?; number }
p(held)
