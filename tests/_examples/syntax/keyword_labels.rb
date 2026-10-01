# A keyword written as a label is a name like any other.
options = { if: :ready, next: 2, class: "primary", end: 10, self: true, not: false }
p(options)
p(options[:next] + options[:end])

def schedule(if: true, until: 5, in: "UTC")
  [binding.local_variable_get(:if), binding.local_variable_get(:until), binding.local_variable_get(:in)]
end
p(schedule)
p(schedule(until: 9, in: "CET"))

# Keywords name symbols too.
p([:not, :and, :or, :in, :__FILE__, :__LINE__, :__dir__, :__ENCODING__, :defined?])

# A ternary keeps its colon.
ready = false
p(ready ? nil:0)

# Shift assignments read inside an endless method body.
def halve(count) = count >>= 1
def quadruple(count) = count <<= 2
p(halve(40))
p(quadruple(3))
