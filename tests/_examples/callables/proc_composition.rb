# Proc#>> reads left to right and Proc#<< reads right to left. The composition
# is strict about its arguments when the callable reached first is.
square = proc { |number| number * number }
double = proc { |number| number + number }

p (square >> double).call(3)
p (square << double).call(3)

triple = Object.new
def triple.call(number)
  number * 3
end

inc = proc { |number| number + 1 }
p (inc >> triple).call(3)
p (inc << triple).call(3)

strict = lambda { |number| number - 1 }
p (strict >> inc).lambda?
p (inc >> strict).lambda?
p (inc << strict).lambda?

begin
  inc >> Object.new
rescue TypeError => trouble
  p trouble.message
end
