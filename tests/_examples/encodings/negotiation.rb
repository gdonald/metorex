# `Encoding.compatible?` answers the encoding two objects can be read in
# together, and a pattern reports the encoding it is read in.
p Encoding.compatible? "abc", "def"
p Encoding.compatible? "abc".dup.force_encoding("us-ascii"), "あ"
p Encoding.compatible? "あ", "\xff".dup.force_encoding("binary")
p Encoding.compatible? "abc", "1234".dup.force_encoding("utf-16le")
p Encoding.compatible? Encoding::UTF_8, Encoding::US_ASCII
p Encoding.compatible? Object.new, "abc"

p(/abc/.encoding)
p(/\u{3042}/.encoding)
p(/abc/n.encoding)
p(/\xc2\xa1/n.encoding)
p(/abc/u.encoding)

japanese = "あ".encode "euc-jp"
p Regexp.new(japanese).encoding
