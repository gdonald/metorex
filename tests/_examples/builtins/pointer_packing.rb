# `pack` with 'P' or 'p' writes where a string stands rather than what it
# holds, and `unpack` reads the string back out of it. Only a string that
# `pack` built, or a copy of one, carries an address into.
packed = ["hello"].pack "P"
p packed.size == [0].pack("J").size
p packed.unpack("P5")
p packed.unpack("P1")
p packed.unpack("P10")
p packed.dup.unpack("P5")

# 'p' reads the whole run rather than a count of characters.
held = ["hello"].pack "p"
p held.unpack("p")

# Nothing packs as a null pointer.
p [nil].pack("P").unpack("J")

# A string that never carried one has nothing to read back.
begin
  packed.to_sym.to_s.unpack "P5"
rescue ArgumentError => problem
  p problem.message
end

# Quoted-printable and base64 write the bytes as plain text.
p ["a b"].pack("M")
p ["\t\n"].pack("M")
p ["abcdefghi"].pack("M3")
p ["abc"].pack("m")
p ["abcdefg"].pack("m3")
p [""].pack("m")
