# A multiple assignment may group its targets, and a group takes its value
# apart the way the whole list does. A splat with no name after it takes the
# values the other targets leave and keeps none of them.
(first, second), third = [1, 2], 3
p [first, second, third]

((a, b), c), (d, (e,), (f, (g, h))) = 1
p [a, b, c, d, e, f, g, h]

(only, *rest) = [4, 5, 6]
p [only, rest]

(*taken) = nil
p taken

held, (left, right) = 7, [8, 9]
p [held, left, right]

# A subscript naming more than one index is a target too, which is how a pair
# of slices trade places.
numbers = [1, 2, 3, 4]
numbers[0, 2], numbers[2, 2] = numbers[2, 2], numbers[0, 2]
p numbers

# Constants under a module are targets the same way names are.
holder = Module.new
holder::First, holder::Second = :a, :b
p [holder::First, holder::Second]

# A condition may name what it reads, and a `do` after it opens the body
# rather than a block on the value.
countdown = [:one, :two]
walked = []
while (taking = countdown.shift) do
  walked << taking
end
p walked

# `and` and `or` join the tests a modifier reads.
never = (1 while false or false)
p never
once = (2 if true and true)
p once
