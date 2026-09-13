# A string counts its characters the way the encoding it is tagged with reads
# them, so re-tagging the same bytes changes the count.
held = +"こにちわ"
p(held.length)
p(held.bytesize)
p(held.force_encoding("BINARY").length)

mixed = "あ" + "a"
p(mixed.encoding)
p(mixed.bytesize)
p(mixed.length)
mixed.force_encoding(Encoding::ASCII_8BIT)
p(mixed.length)

# A fixed-width encoding reads whole units, and a unit left short at the end
# still counts as the one broken character it spells.
p("\x00\xd8".dup.force_encoding("UTF-16LE").length)
p("\xd8\x00".dup.force_encoding("UTF-16BE").length)
p("\x04\x03\x02\x01".dup.force_encoding("UTF-32LE").length)

# Each invalid byte in UTF-8 counts as one character.
p("\xF4\x90\x80\x80".length)
p("a\xF4\x90\x80\x80b".length)
