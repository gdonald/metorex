# `allbits?` asks whether every bit of the mask is set, `anybits?` whether any
# is, and `nobits?` whether none is.
p(42.allbits?(42))
p(0b1010_1010.allbits?(0b1000_0010))
p(0b1010_1010.allbits?(0b1000_0001))

p(0b1010_1010.anybits?(0b1000_0001))
p(0b1010_1010.anybits?(0b0101_0101))

p(0b1010_1010.nobits?(0b0101_0101))
p(0b1010_1010.nobits?(0b1000_0001))

# Negative numbers compare in two's complement, and a number past the fixnum
# range compares the same way.
complement = ~0b1
p(complement.allbits?(42))

negative = -42
p(negative.allbits?(negative))

huge = 2 ** 64
left = 0b1010_1010 | huge
p(left.allbits?(0b1000_0010 | huge))
p(left.nobits?(0b0101_0101))

# The mask is asked for `to_int`.
class BitMask
  def to_int
    0b10
  end
end

mask = BitMask.new
p(0b110.allbits?(mask))

begin
  13.allbits?("10")
rescue TypeError => error
  puts(error.message)
end
