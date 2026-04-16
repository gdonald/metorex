# Numeric methods demonstration (with parentheses)

# Integer methods
puts(42.abs)
x = -7
puts(x.abs)

puts(42.to_f)
puts(42.to_s)
puts(42.to_i)

# Integer#times
sum = 0
5.times { |i| sum += i }
puts(sum)

# Float methods
puts(3.14.abs)
y = -2.5
puts(y.abs)

puts(3.7.ceil)
puts(3.2.floor)
puts(3.14159.round(2))

puts(3.7.to_i)
puts(3.14.to_f)
puts(3.14.to_s)
