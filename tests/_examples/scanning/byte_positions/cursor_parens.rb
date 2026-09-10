require "strscan"

# The cursor a scanner reports counts bytes, so a character made of several
# bytes moves it by that many.
scanner = StringScanner.new("abcädeföghi")
wanted = /ö/
scanner.scan_until(wanted)
p(scanner.pos)
p(scanner.pointer)
p(scanner.charpos)

scanner.pos = 3
p(scanner.pos)
p(scanner.rest)

# A byte read leaves the character it read as the match.
plain = StringScanner.new("This is a test")
p(plain.get_byte)
p(plain[0])
p(plain.matched?)

# `bytes` with a block hands over each byte and answers the string.
counted = []
p("東京".bytes { |byte| counted << byte })
p(counted)
