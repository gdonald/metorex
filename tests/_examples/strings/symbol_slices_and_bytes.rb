# Symbol#[] reads the name the way a String does: a Float index is read
# through to_int, a span past the end names nothing, and a pattern records
# the match so $~ names it afterwards.
p(:symbol[1.5])
p(:symbol[2, 2.8])
p(:symbol[6..])
p(:symbol[7..])
p(:symbol[/(sy)(mb)/, 2])
p(:symbol[/(sy)(mb)/, 4])
:symbol[/mb/]
p($~[0])

# String#byteslice cuts by byte rather than by character.
p("héllo".byteslice(0, 3))
p("héllo".byteslice(1..2))
p("héllo".byteslice(99))

# append_as_bytes puts bytes on the end without reading them, so the string
# keeps the encoding it had however broken the result is.
held = +"hello"
held.append_as_bytes("\xE2\x82")
p(held.valid_encoding?)
held.append_as_bytes("\xAC")
p(held.valid_encoding?)
p(held)

counted = "".b
counted.append_as_bytes(0x131, -1, "ab")
p(counted.bytes)
