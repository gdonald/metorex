require "stringio"
io = StringIO.new("")
io.write "abc"
printf(io, "%d-%s", 5, "x")
p io.string
