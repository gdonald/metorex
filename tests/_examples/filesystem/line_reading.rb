# Lines are read up to a separator, or up to a count of bytes, and a stream
# asked for a paragraph reads to the blank line that ends one.
require "tmpdir"

path = File.join Dir.tmpdir, "metorex_line_reading.txt"
File.write path, "one two\nthree four\n\nfive six\n"

stream = File.open path, "r"
p stream.gets
p stream.gets " "
p stream.gets 4
p stream.gets("\n", chomp: true)
p $.
stream.close

stream = File.open path, "r"
p stream.gets ""
p stream.gets ""
stream.close

stream = File.open path, "r"
p stream.readline chomp: true
p stream.readlines chomp: true
stream.close

stream = File.open path, "r"
walked = []
stream.each_line(chomp: true) { |line| walked << line }
p walked
stream.close

# A character put back is read again before anything the stream has left,
# and `sysread` refuses a stream whose lines have been read.
stream = File.open path, "r"
letter = stream.getc
stream.ungetc letter
p stream.readpartial 3
p stream.read 2
begin
  stream.sysread 2
rescue IOError => trouble
  p trouble.message
end
stream.close

stream = File.open path, "r"
p stream.sysread 3
p stream.readpartial 4, +"held"
stream.close

File.delete path
