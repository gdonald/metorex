require "stringio"

# A buffer handed to `read` is filled with what was read and answered in its
# place, so the caller keeps the one string it passed.
stream = StringIO.new("this is\nan example\n")
buffer = +""
p(stream.read(4, buffer).equal?(buffer))
p(buffer)

stream.rewind
p(stream.gets)
p($_)
p(stream.gets(chomp: true))

stream.rewind
p(stream.readlines(" "))
stream.rewind
p(stream.each_line.to_a)

# Truncating cuts the buffer itself.
text = +"123456789"
held = StringIO.new text
held.truncate(4)
p(text)

# A stream reports the encoding its buffer carries.
tagged = StringIO.new(+"abc")
tagged.set_encoding(Encoding::US_ASCII)
p(tagged.external_encoding)
p(tagged.string.encoding)
