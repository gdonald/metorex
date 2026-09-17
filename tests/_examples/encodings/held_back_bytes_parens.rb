# encoding: binary
# A run of bytes the source encoding cannot read stops the conversion. The
# bytes after the run are held back, and `putback` hands them over so the
# next piece of text can start with them.
held = Encoding::Converter.new("EUC-JP", "ISO-8859-1")
source = +"abc\xa1def"
destination = +""

p(held.primitive_convert(source, destination, nil, 10))
p(held.primitive_errinfo)
p(held.putback)
p(held.putback)
p(held.primitive_errinfo.last)

# An encoding written two bytes at a time is cut at those boundaries, so the
# run that could not be read is the whole pair and so is what follows it.
wide = Encoding::Converter.new("utf-16le", "iso-8859-1")
p(wide.primitive_convert(+"\x00\xd8\x61\x00", +""))
p(wide.primitive_errinfo)
p(wide.putback.bytes)
