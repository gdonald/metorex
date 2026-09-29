# readpartial and read_nonblock hand back what is already there, including
# what ungetc put back, without waiting for more.
reader, writer = IO.pipe
writer.write("foobar")
character = reader.getc
reader.ungetc(character)
p(reader.readpartial(3))
p(reader.readpartial(3))
reader.ungetc("w")
p(reader.read_nonblock(5))
p(reader.read_nonblock(5, exception: false))
