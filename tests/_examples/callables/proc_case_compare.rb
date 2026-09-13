# Proc#=== calls the proc with the given argument, so a callable can stand in
# for a pattern in a case/when.
positive = proc { |number| number > 0 }
p positive.===(5)
p positive.===(-5)

pair = proc { |first, second| [first, second] }
p pair.===([1, 2])

strict = lambda { |first, second| [first, second] }
p strict.===(1, 2)

begin
  strict.===(1)
rescue ArgumentError => trouble
  p trouble.message
end

def describe(value)
  even = proc { |number| number.even? }
  small = lambda { |number| number < 10 }
  case value
  when even then "even"
  when small then "small and odd"
  else "large and odd"
  end
end

p describe(4)
p describe(7)
p describe(11)
