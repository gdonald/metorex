# One default may set several parameters at once, and a later parameter reads
# what an earlier one was given.
def shared(a = b = c = {})
  [a, b, c]
end

p(shared)
p(shared(1))

def widened(first = 2, second = first * 3)
  [first, second]
end

p(widened)
p(widened(5))
