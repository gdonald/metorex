# `primitive_convert` carries what it can from a source buffer into a
# destination buffer and answers why it stopped, leaving in the source what
# it did not read.
latin = Encoding::Converter.new "utf-8", "iso-8859-1"
source = "á顸abcd"
written = +""
p latin.primitive_convert(source, written)
p source
p written.bytes
p written.encoding

# Output past the destination bytesize is kept and written first next time.
held = Encoding::Converter.new "utf-8", "iso-8859-1"
buffer = +"aa"
p held.primitive_convert("b", buffer, nil, 0)
p held.primitive_convert("b", buffer, nil, 1)
p held.primitive_convert("b", buffer, nil, 2)
p buffer

# The byte offset names where writing starts, and cannot pass the end.
padded = +"   "
p held.primitive_convert("abc", padded, 2)
p padded
p((held.primitive_convert("", +"am", 3) rescue $!.message))

# The byte that cut an invalid run short is read again on the next call.
again = Encoding::Converter.new "utf-8", "iso-8859-1"
broken = "\xf1abcd".b
out = +""
p again.primitive_convert(broken, out)
p broken
p again.primitive_errinfo
p again.primitive_convert(broken, out)
p out

# A lone continuation byte is wrong by itself.
lone = Encoding::Converter.new "utf-8", "iso-8859-1"
bytes = "\xC3\xA1\x80\x80".b.force_encoding "utf-8"
p lone.primitive_convert(bytes, +"")
p bytes.b

# With `partial_input: true`, a character the input stops part-way through
# waits for the rest.
waiting = Encoding::Converter.new "EUC-JP", "UTF-8"
start = "\xa4".b
p waiting.primitive_convert(start, +"", nil, nil, partial_input: true)
p start
finished = +""
p waiting.primitive_convert("\xa2".b, finished)
p finished

# ISO-2022-JP ends a finished conversion back in ASCII.
jis = Encoding::Converter.new "utf-8", "iso-2022-jp"
jis_out = +""
p jis.primitive_convert("香", jis_out, 0, 2)
p jis.primitive_convert("", jis_out)
p jis_out.bytes
p Encoding::UTF_8_MAC

# EUC-JP reads a lead byte together with the ones after it.
euc = "\xa4\xa2a".b.force_encoding "EUC-JP"
p euc.length
p euc.each_char.map(&:bytes)

# `replace` takes the bytes of what it is handed along with its encoding.
replaced = +""
replaced.replace "\xE1".b.force_encoding("ISO-8859-1")
p replaced.bytes
