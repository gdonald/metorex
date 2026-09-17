# `chr` names a character by its code. Written with no encoding, a code in the
# ASCII range is ASCII and one above it is a byte.
p(65.chr.encoding)
p(200.chr.encoding)
p(200.chr.bytes)

# An encoding named outright decides how the code is read: a code point in a
# Unicode encoding, a byte in a single-byte one, and the bytes themselves in
# one that spells characters with several.
p(0x3042.chr(Encoding::UTF_8).bytes)
p(0xA4A2.chr("euc-jp").bytes)
p(0x8140.chr(Encoding::SHIFT_JIS).bytes)
p(0x10400.chr(Encoding::CESU_8).bytes)

# A code the encoding has no character for is refused.
[[0x100, "US-ASCII"], [0x100, "BINARY"], [0xA1A0, "EUC-JP"], [0x80, "SHIFT_JIS"],
 [0x100, "ISO-8859-9"], [620, "TIS-620"], [0xD800, "UTF-8"]].each do |code, named|
  begin
    code.chr(named)
    p([code, named, "built"])
  rescue RangeError
    p([code, named, "refused"])
  end
end

# `byteslice` answers nil where the byte it names is not there, and a bound
# too wide for a machine word is refused.
p("hello".byteslice(5))
p("hello".byteslice(5, 2))
p("".byteslice(0))
begin
  "hello".byteslice(2**64)
rescue RangeError => trouble
  p(trouble.class)
end
