# `Proc#yield` calls the proc the way `call` and `[]` do, taking the same
# arguments and the same block.
doubler = proc { |number| number * 2 }
p doubler.yield 4
p doubler.call 4
p doubler[4]

strict = lambda { |first, second| first - second }
p strict.yield 9, 4

with_block = proc { |&given| given.call 7 }
p with_block.yield { |number| number + 1 }
