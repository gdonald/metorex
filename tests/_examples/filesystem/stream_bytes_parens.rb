# A stream is read a byte and a character at a time, what is put back is read
# again first, and one told not to sync holds what is written until a flush.
require "tmpdir"

path = File.join(Dir.tmpdir, "metorex_stream_bytes_parens.txt")
File.write(path, "abc")

handle = File.open(path, "r")
p(handle.getbyte)
p(handle.getc)
handle.ungetbyte(122)
p(handle.getbyte)
p(handle.getc)
p(handle.getbyte)
handle.close

handle = File.open(path, "r")
collected = []
handle.each_byte { |byte| collected << byte }
p(collected)
handle.close

handle = File.open(path, "r")
letters = []
handle.each_char { |letter| letters << letter }
p(letters)
p(handle.advise(:sequential))
handle.close

handle = File.open(path, "r+")
p(handle.pread(2, 0))
p(handle.pos)
handle.close

reader, writer = IO.pipe
writer.sync = false
writer.write("held")
writer.flush
p(reader.read(4))
writer.close
reader.close

File.delete(path)
