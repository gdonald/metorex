# A run of numeric escapes names bytes, and bytes that spell a character in
# UTF-8 read back as that character.
p("\343\203\255")
p("\xE3\x83\xAD")
p("\343\203\255".bytes)
p("\000".bytes)
p("a\101b")

# A percent literal reads the same escapes a quoted string does.
p(%[<\xA4??>].length)
p(%(tab\there))
p(%{\343\203\255})
p(%(\e[1m).bytes)
