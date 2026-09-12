# A curried lambda gathers its arguments one call at a time, and stays a
# lambda as it goes.
add = -> first, second, third { first + second + third }
curried = add.curry
p curried.call(1).call(2).call 3
p curried.lambda?

# A plain proc curries into plain procs.
loose = proc { |first, second| [first, second] }
p loose.curry.lambda?
p loose.curry.call(1).call 2

# A Method curries the same way, and stands for a lambda.
class Adder
  def sum first, second
    first + second
  end
end

taken = Adder.new.method :sum
p taken.lambda?
p taken.curry.call(4).call 5

# A Method handed over with & reaches a block, keeping its own arity.
def holding &block
  [block.lambda?, block.arity]
end

p holding(&taken)

# A parameter group spreads one array argument across the names in it.
pairs = -> ((first, (second, third))) { [first, second, third] }
p pairs.call [1, [2, 3]]

# super hands the parent the block the method was called with.
class Counter
  def each
    yield 1
    yield 2
  end
end

class LoudCounter < Counter
  def each
    super
  end
end

seen = []
LoudCounter.new.each { |value| seen.push value }
p seen
