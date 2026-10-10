# The value a `break`, `next` or `return` carries can be an assignment, and
# the variable keeps what was assigned.
found = nil
[3, 8, 12].each do |number|
  break found = number if number > 5
end
p found

doubled = [1, 2].map { |number| next twice = number * 2 }
p doubled

def first_even numbers
  numbers.each { |number| return @even = number if number.even? }
  nil
end
p first_even [1, 4, 6]
p @even
