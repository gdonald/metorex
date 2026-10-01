# How a call hands over its arguments: a splat asks for `to_a`, keywords
# keep keys that are not Symbols and the order they were written in, `**nil`
# refuses keywords, and a space before the parentheses makes them one
# argument.
# Written with as few parentheses as Ruby allows.

def gather(*values)
  values
end

class Refuses
  def to_a
    1
  end
end
begin
  gather(*Refuses.new)
rescue TypeError => error
  p error.message
end

class Declines
  def to_a
    nil
  end
end
p gather(*Declines.new).first.class

def options(**given)
  given
end
p options("name" => 1, size: 2)
p options(**{a: 1, b: 2}, **{a: 4, c: 7})

def required_and_rest(a:, **rest)
  [a, rest]
end
p required_and_rest("a" => 1, a: 1, b: 2)

def takes_no_keywords(value, **nil)
  value
end
p takes_no_keywords({a: 1})
begin
  takes_no_keywords(a: 1)
rescue ArgumentError => error
  p error.message
end

p gather ()
p gather ((0; 1)), ((2; 3))
begin
  eval "gather (1, 2)"
rescue SyntaxError
  p SyntaxError
end

def pair(first:, second:)
  [first, second]
end
first = 1
second = 2
p pair first:, second:
