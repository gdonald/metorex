# A stream over a list of names reads the files as though they were one. It
# walks bytes, characters and codepoints, reads a count into a buffer, and
# reads a file at a time when asked for part of one.
require "tmpdir"

first = File.join Dir.tmpdir, "metorex_argf_one.txt"
second = File.join Dir.tmpdir, "metorex_argf_two.txt"
File.write first, "one\n"
File.write second, "two\n"

walk = ARGF.class.new first, second
p walk.each_byte.first(4)
walk.close

walk = ARGF.class.new first, second
letters = []
walk.each_char { |letter| letters << letter }
p letters

walk = ARGF.class.new first, second
p walk.each_codepoint.to_a.length

walk = ARGF.class.new first, second
lines = []
walk.each_line { |line| lines << line }
p lines

walk = ARGF.class.new first, second
p walk.gets
p walk.filename == first
p walk.gets
p walk.gets

# A count reads from the file being read and stops there, and what is read
# lands in the buffer when one is handed over.
walk = ARGF.class.new first, second
buffer = +"held"
p walk.read(4, buffer)
p buffer
p walk.readpartial 4
p walk.readpartial 4

# The encodings the files are read as.
walk = ARGF.class.new first
walk.set_encoding "us-ascii"
p walk.external_encoding.to_s
p walk.gets.encoding.to_s

File.delete first
File.delete second
