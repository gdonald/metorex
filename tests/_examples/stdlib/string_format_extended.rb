# Format specifier: %i (alias for %d)
puts "%i" % 42

# Plus sign flag
puts "%+d" % 42
puts "%+d" % -5

# Space sign flag
puts "% d" % 42
puts "% d" % -5

# Float with plus/space
puts "%+f" % 3.14
puts "% f" % 3.14

# Integer from float
puts "%d" % 3.99

# Float plus sign for int arg
puts "%+d" % 3.5
puts "% d" % 3.5

# Hex uppercase
puts "%X" % 255

# Octal
puts "%o" % 8

# Binary
puts "%b" % 10

# Char from int
puts "%c" % 65

# Char from string
puts "%c" % "hello"

# Inspect nil
puts "%p" % nil

# Inspect number
puts "%p" % 42

# String precision
puts "%.3s" % "hello"

# Right-align with width
puts "%10s|" % "right"

# Zero-pad right-align
puts "%010d" % 42

# Trailing percent at end of format
puts "100%"
