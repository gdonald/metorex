# A string splits into characters the way the encoding it is tagged with
# reads them, and each character carries that same encoding.
held = +"\u{24B62}"
p(held.chars)
p(held.chars.map { |character| character.encoding.name })
p(held.dup.force_encoding("BINARY").chars)
p(held.dup.force_encoding("SJIS").chars.map { |character| character.bytes })

# An encoding Ruby names without converting anything through it has one
# character to the byte.
p("abcd".dup.force_encoding(Encoding::UTF_16).chars.length)
p("ab".dup.force_encoding(Encoding::UTF_7).chars.map { |character| character.bytes })

# Given a block, `chars` hands each character over and answers the string.
greeting = +"hello"
seen = []
p(greeting.chars { |character| seen << character }.equal?(greeting))
p(seen)

# A method that answers the string it was called on answers the subclass
# instance, not the characters behind it.
class Spelled < String
end
subject = Spelled.new("hello")
p(subject.each_char {}.equal?(subject))

# Text converts between UTF-8 and a single-byte encoding by its characters
# rather than by its bytes.
euro = "\u{20AC}".dup.force_encoding("UTF-8")
latin = euro.encode("ISO-8859-15")
p(latin.bytes)
p(latin.encoding)
p(latin.encode("UTF-8"))
