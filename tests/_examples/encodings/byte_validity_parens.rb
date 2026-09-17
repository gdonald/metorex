# Whether a run of bytes spells characters depends on the encoding it is read
# in. The same three bytes read as UTF-8, as bytes, or as a single-byte
# encoding are all fine, while an encoding that pairs its bytes refuses them.
held = "\xE6\x9D\x94".dup
p(held.force_encoding("UTF-8").valid_encoding?)
p(held.force_encoding("BINARY").valid_encoding?)
p(held.force_encoding("ISO-8859-9").valid_encoding?)
p(held.force_encoding("US-ASCII").valid_encoding?)
p(held.force_encoding("Shift_JIS").valid_encoding?)
p(held.force_encoding("EUC-JP").valid_encoding?)
p(held.force_encoding("Big5").valid_encoding?)

# A half of a surrogate pair standing alone spells no character, and a fixed
# width encoding reads whole units.
p("\xD8\x00".dup.force_encoding("UTF-16BE").valid_encoding?)
p("\x04\x03\x02\x01".dup.force_encoding("UTF-32BE").valid_encoding?)

# An encoding Ruby names without converting through it reads its text a byte
# at a time, so every run spells characters in it.
p("abcd".dup.force_encoding("UTF-16").valid_encoding?)
p(held.force_encoding("UTF-7").valid_encoding?)

# Text that stands for bytes carries that over when it is appended, so a byte
# the encoding cannot read stays the byte it was.
built = +"a"
built << [0xDD].pack("C").force_encoding("utf-8")
p(built.bytes)
p(built.valid_encoding?)
