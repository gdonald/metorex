# %x with non-int (fallback to to_s)
puts "%x" % "hello"

# %X with non-int
puts "%X" % "hello"

# %o with non-int
puts "%o" % "hello"

# %b with non-int
puts "%b" % "hello"

# %c with int
puts "%c" % 65

# %c with string
puts "%c" % "Z"

# Width with right-align (non-zero-pad)
puts "%8s!" % "hi"

# Integer format from float
puts "%i" % 7.9

# Float with int arg
puts "%f" % 5

# Float with precision and int
puts "%.2f" % 5

# Trailing % at end (no specifier)
puts "test%"
