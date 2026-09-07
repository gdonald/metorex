# A String reports its bytes as well as its characters.
text = "héllo"

p text.length
p text.bytesize
p text.bytes
p text.getbyte(0)
p text.getbyte(1)
p text.getbyte(-1)
p text.getbyte(99)
p text.chr
p text.ascii_only?
p "hello".ascii_only?
p text.valid_encoding?

collected = []
"abc".each_byte { |byte| collected.push(byte) }
p collected
p "abc".each_byte.to_a

# hex reads a number off the front in base 16, honoring only the 0x prefix.
p "0a".hex
p "0x1f".hex
p "A_BAD_BABE".hex
p "0b1010".hex
p "abcdefG".hex
p "not a number".hex
p "-1234".hex

# oct reads base 8 by default and honors every base prefix.
p "777".oct
p "0b1010".oct
p "0o17".oct
p "0d99".oct
p "0xff".oct
p "-777".oct
p "8".oct
