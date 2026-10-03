# A Symbol spelled in ASCII from an ASCII-compatible encoding reads as
# US-ASCII, one spelling anything else keeps its string's encoding, and one
# from an encoding that is not ASCII-compatible keeps that encoding even
# when it spells only ASCII.
p "plain".to_sym.encoding
p "café".to_sym.encoding
wide = "test_symbol".encode(Encoding::UTF_16LE).to_sym
p wide.encoding
p wide.to_s.encoding
p wide.to_s.bytesize
