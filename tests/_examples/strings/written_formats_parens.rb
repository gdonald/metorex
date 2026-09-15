# `sprintf` writes a value the way a directive names: a base, a fraction, a
# character, or the text a value stands for.
p(sprintf("%b %o %x %X", 10, 87, 196, 196))
p(sprintf("%d %i %u", 112, 112, 112))
p(sprintf("%e %E %f", 109.52, 109.52, 10.952))
p(sprintf("%g %G", 0.0000123456, 1234567))
p(sprintf("%a %A", 196, 196))
p(sprintf("%c %p %s", 97, [1], "abc"))

# A negative number written in a base of its own carries a run of the base's
# largest digit reaching back forever.
p(sprintf("%b %o %x %X", -10, -87, -196, -196))
p(sprintf("%010b %.7b", -10, -5))

# A sign, a width, a precision, and the `#` that writes the base out.
p(sprintf("%+d % d %-6d| %06d", 5, 5, 5, 5))
p(sprintf("%#b %#o %#x %#X", 10, 87, 196, 196))
p(sprintf("%.3s %6.2f %*d", "hello", 3.14159, 6, 42))

# A directive can name which value it takes and read a width from another.
p(sprintf("%2$s %1$s", "world", "hello"))
p(sprintf("%1$*2$d", 42, 8))

# A name reads its value from a Hash instead of from a position.
p(sprintf("%<count>05d and %{label}", count: 42, label: "rest"))

# Infinity and NaN are named rather than written out.
p(sprintf("%f %e %g", Float::INFINITY, -Float::INFINITY, Float::NAN))
