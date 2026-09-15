# A String handed to a number directive is read the way `Integer()` reads one,
# so the base a prefix names is the one it counts in.
puts("%x" % "196")
puts("%X" % "0xc4")
puts("%o" % "0127")
puts("%b" % "0b1010")

# A String naming no number at all is refused.
begin
  "%x" % "hello"
rescue ArgumentError => trouble
  puts(trouble.message)
end

# %c names a character by its code, or takes the first of a string.
puts("%c" % 65)
puts("%c" % "Z")

# Width with right-align (non-zero-pad)
puts("%8s!" % "hi")

# Integer format from float
puts("%i" % 7.9)

# Float with int arg
puts("%f" % 5)

# Float with precision and int
puts("%.2f" % 5)

# Trailing % is written by doubling it
puts("test%%" % [])
