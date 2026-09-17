# `pack("U")` writes a code point in UTF-8, carrying on past the range Unicode
# names the way Ruby does, and the answer is tagged UTF-8.
p([0x41, 0x3042].pack("U*"))
p([0x41, 0x3042].pack("U*").encoding)
p([0x00110000].pack("U").bytes)
p([0x7FFFFFFF].pack("U").bytes)
begin
  [2**32].pack("U")
rescue RangeError => trouble
  p(trouble.class)
end

# Uuencoding and base64 answer ASCII, and a count names how many bytes one
# uuencoded line holds, rounded down to whole groups of three.
p(["abcdefghijklm"].pack("u7"))
p(["abc"].pack("u").encoding)
p(["abc"].pack("m").encoding)
p([65].pack("C").encoding)

# Reading back finds the code points the bytes spell, and bytes that spell no
# character are refused.
p("あ".unpack("U"))
begin
  "\xE3".unpack("U")
rescue ArgumentError => trouble
  p(trouble.class)
end

# A width modifier belongs only to a directive with a platform width.
begin
  "abc".unpack("a!")
rescue ArgumentError => trouble
  p(trouble.message)
end
