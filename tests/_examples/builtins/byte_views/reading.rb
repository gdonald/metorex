require "stringio"
require "io/console"

# A stream counts its cursor in bytes, so a character made of several bytes
# moves it by that many.
stream = StringIO.new "föóbar"
stream.getch
p stream.pos
stream.getch
p stream.pos
stream.seek 3, IO::SEEK_END
p stream.pos
p stream.external_encoding
stream.binmode
p stream.external_encoding
stream.set_encoding Encoding::EUC_JP
p stream.external_encoding

# `unpack` tags what it reads: text as bytes, bits and nibbles as ASCII.
p "abc".unpack("a*").first.encoding
p "abc".unpack("B8").first.encoding
p "abc".unpack("H2").first.encoding
p [0xFF].pack("C").getbyte(0)

# A number written as a Rational has to read back as one.
class Whole
  def to_r
    1
  end
end
begin
  Rational Whole.new
rescue TypeError => problem
  p problem.message
end
