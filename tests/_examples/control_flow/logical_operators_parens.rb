# Logical operators: && and || (with parentheses)
x = true
y = false

puts(x && y)
puts(x || y)
puts(false && true)
puts(false || true)

# Short-circuit: || returns right side when left is falsy
a = nil || 42
puts(a)

# Short-circuit: && returns left side when left is falsy
b = nil && 42
puts(b)

# Chaining
puts(true && true && true)
puts(true && false && true)
puts(false || false || true)
