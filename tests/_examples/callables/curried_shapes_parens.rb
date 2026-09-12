# A curried callable stands for the library's own: it names no arguments of
# its own and has no scope behind it. A hash may take a callable as its
# default, and anything that spells itself out as one will do.
add = proc { |a, b| a + b }
curried = add.curry
p(curried.parameters)
p(curried.arity)
p(curried[1][2])

begin
  curried.binding
rescue ArgumentError => trouble
  p(trouble.class)
end

strict = lambda { |a, b, c| a + b + c }
p(strict.curry[1][2][3])
p(strict.curry.parameters)

class Spelled
  def to_proc
    proc { |held, key| "#{key} is missing" }
  end
end

counted = Hash.new(42)
p(counted[:nothing])
counted.default_proc = Spelled.new
p(counted.default)
p(counted[:city])

counted.default_proc = nil
p(counted.default_proc)
p(counted[:town])

begin
  counted.default_proc = 42
rescue TypeError => trouble
  p(trouble.class)
end

begin
  counted.default_proc = lambda { |only| only }
rescue TypeError => trouble
  p(trouble.class)
end
