# A number written in another base carries the same `r` and `i` suffixes a
# decimal one does, and the value it stands for is the whole number it named.
p 0xffr
p 042r
p 0b1111r
p(-0xffr)

p 0xffi
p 042i
p 0b1110i

# A decimal with a fraction is exactly the fraction it was written as,
# rather than the binary float nearest it.
p 0.3r
p 0.0174532925199432957r

# A literal past what fits in a machine word keeps every digit.
p 1111111111111111111111111111111111111111111111r
