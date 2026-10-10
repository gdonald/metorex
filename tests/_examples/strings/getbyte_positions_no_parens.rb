plain = "rate"
p [plain.getbyte(0), plain.getbyte(-1), plain.getbyte(4), plain.getbyte(-5)]

accented = "café"
p [accented.getbyte(3), accented.getbyte(4), accented.getbyte(-1), accented.getbyte(5)]

packed = [7, 200, 255].pack("C*")
p [packed.getbyte(0), packed.getbyte(1), packed.getbyte(-1), packed.getbyte(-4), packed.getbyte(3)]

wide = "a€".encode("UTF-16LE")
p [wide.getbyte(0), wide.getbyte(2), wide.getbyte(-1), wide.getbyte(4)]

binary = "é".b
p [binary.getbyte(0), binary.getbyte(1), binary.getbyte(-2)]

require "stringio"
stream = StringIO.new([1, 0, 200, 9].pack("C*"))
p [stream.getbyte, stream.getbyte, stream.pos]
stream.pos -= 1
p [stream.getbyte, stream.read(1).bytes, stream.pos, stream.getbyte]

text = StringIO.new("añb")
text.pos = 3
p [text.pos, text.getc]
