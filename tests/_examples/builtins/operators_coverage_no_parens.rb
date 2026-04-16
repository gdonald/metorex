# Integer overflow to Float (no parens where possible)
big = 4611686018427387903
puts big + big
puts big * 3
puts 0 - big - big - 2

# String comparisons
puts "abc" < "abd"
puts "abd" > "abc"
puts "abc" <= "abc"
puts "abc" >= "abc"
puts "abc" <= "abd"
puts "abd" >= "abc"

# Bitwise on integers
puts 5 & 3
puts 5 | 3
puts 5 ^ 3
