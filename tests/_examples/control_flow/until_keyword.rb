# `until` keyword — loops while condition is false.
# Covers both prefix `until cond; body; end` and postfix `body until cond`.

# Prefix form: count up from 0 to 3.
i = 0
until i == 3
  puts i
  i = i + 1
end

# Postfix modifier: same loop on one line.
j = 0
j = j + 1 until j == 5
puts j

# Until with a compound condition.
n = 10
until n <= 0 || n == 7
  n = n - 1
end
puts n
