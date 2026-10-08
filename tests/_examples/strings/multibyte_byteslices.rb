# A byte slice of UTF-8 text that holds whole characters is text again, and
# prints and joins as the characters it holds.
text = "naïve café ─ done"
word = text.byteslice 0, 6
puts word
p word, word.length, word.encoding, word.valid_encoding?
joined = word + " plan"
puts joined
p joined.length
rule = text.byteslice 13..15
puts "[#{rule}]"
p rule.bytes
half = text.byteslice 2, 1
p half, half.valid_encoding?, half.length
marked = "\uFEFF# heading\n"
print marked.byteslice(0, 3) + marked.byteslice(3..)
p((marked.byteslice(0, 3) + "x").bytes)
