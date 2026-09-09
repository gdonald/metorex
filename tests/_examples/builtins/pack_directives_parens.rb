# Every directive the pack machine reads, written out and read back.

# Text, padded three ways.
p(["ab"].pack("a4").length)
p(["ab"].pack("A4"))
p(["ab"].pack("Z4").length)
p("abcd".unpack("a2a2"))
p("ab  ".unpack("A4"))
p("ab\x00d".unpack("Z*a*"))

# Bits and nibbles, each end first.
p(["1010"].pack("B4").bytes)
p(["1010"].pack("b4").bytes)
p(["8f"].pack("H2").bytes)
p(["f8"].pack("h2").bytes)
p("\xa0".unpack("B4"))
p("\xa0".unpack("b4"))
p("\x8f".unpack("H*"))
p("\x8f".unpack("h*"))

# The integer widths, both byte orders, signed and not.
p([1, 2].pack("C*").bytes)
p([-1].pack("c").bytes)
p([258].pack("n").bytes)
p([258].pack("v").bytes)
p([258].pack("N").bytes)
p([258].pack("V").bytes)
p([1].pack("s>").bytes)
p([1].pack("l<").length)
p([1].pack("q>").length)
p([1].pack("j").length)
p("\xff".unpack("c"))
p("\xff".unpack("C"))
p("\x01\x02".unpack("n"))
p("\x01\x02\x03\x04".unpack("N"))
p("\x01\x02\x03\x04\x05\x06\x07\x08".unpack("Q>"))

# Floats, both widths and both orders.
p([1.5].pack("e").length)
p([1.5].pack("E").length)
p([1.5].pack("g").length)
p([1.5].pack("G").length)
p([1.5].pack("f").length)
p([1.5].pack("d").length)
p([1.5].pack("e").unpack("e"))
p([1.5].pack("G").unpack("G"))

# Code points, BER integers, and the placement directives.
p([233].pack("U").length)
p([233].pack("U").unpack("U"))
p([300].pack("w").bytes)
p([300].pack("w").unpack("w"))
p([1].pack("Cx2").length)
p([1, 2].pack("CXC").bytes)
p([1].pack("C@4").length)
p("abcdef".unpack("C@4C"))
p("abcdef".unpack("x2C"))

# A count, a star, and whitespace and comments between directives.
p("abcdef".unpack("C3"))
p("abcdef".unpack("C*").length)
p("abcdef".unpack("C \t C"))
p("abcdef".unpack("C # a note\nC"))

# What the machine refuses.
[
  -> { [1].pack("K") },
  -> { "abc".unpack("K") },
  -> { "abc".unpack("a!") },
  -> { [].pack("N") },
  -> { ["text"].pack("C") },
  -> { "ab".unpack("x4C") },
  -> { "abcd".unpack("CX*C") },
  -> { "ab".unpack("@4C") },
].each do |attempt|
  begin
    attempt.call
  rescue StandardError => problem
    p [problem.class, problem.message]
  end
end

# The encoded forms written out, and read back to what they stood for.
p(["ABC"].pack("m"))
p(["ABC"].pack("m").unpack("m"))
p(["ab\ncd"].pack("M"))
p(["ABC"].pack("u"))
p(["ABC"].pack("u").unpack("u"))
p([960].pack("U*").length)
p([1, 2].pack("U*").unpack("U*"))

# The encoded forms, each read back to the bytes it stands for.
p("QUJD".unpack("m"))
p("QUJD\n".unpack("m"))
p("QQ==".unpack("m"))
p("ab=3Dcd".unpack("M"))
p("ab=\ncd".unpack("M"))
p("ab=ZZ".unpack("M"))
p("#0V%C\n".unpack("u"))
p("".unpack("u"))

# A wide character packs as the bytes its encoding needs.
p([960].pack("U").length)
p([960].pack("U").unpack("U"))
p("abc".unpack("U*"))

# A width modifier is refused on a directive that has no platform width.
begin
  "abc".unpack("a_")
rescue ArgumentError => problem
  p problem.message
end

# A count too wide to hold is refused rather than wrapped.
begin
  "abc".unpack("C99999999999999999999999")
rescue RangeError => problem
  p problem.message
end

# A name that answers to_str stands for the format.
class FormatName
  def to_str
    "C*"
  end
end
p("abc".unpack(FormatName.new))
p([65].pack(FormatName.new))
