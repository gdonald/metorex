# Compressed streams: the two checksums zlib keeps, the encoding underneath,
# and the gzip wrapper that carries it.

require 'zlib'
require 'stringio'

p(Zlib.crc32 "")
p(Zlib.crc32 "123456789")
p(Zlib.adler32 "123456789")
p(Zlib.crc_table.length)

held = "the quick brown fox " * 20
p(Zlib.inflate(Zlib.deflate(held)) == held)
p(Zlib.gunzip(Zlib.gzip(held)) == held)

# A stream written by any other zlib reads back here.
written = [120, 156, 99, 96, 128, 1, 0, 0, 10, 0, 1].pack "C*"
p(Zlib.inflate(written) == "\000" * 10)

# A gzip member carries the name and time it was written with.
member = [31, 139, 8, 0, 44, 220, 209, 71, 0, 3, 51, 52, 50, 54, 49, 77,
          76, 74, 78, 73, 5, 0, 157, 5, 0, 36, 10, 0, 0, 0].pack "C*"
reader = Zlib::GzipReader.new StringIO.new(member)
p(reader.read)
p(reader.eof?)
reader.close

# The reader walks what it holds a line or a character at a time.
lines = Zlib::GzipReader.new StringIO.new(Zlib.gzip("one\ntwo\n"))
p(lines.readlines)
p(Zlib::GzipReader.new(StringIO.new(Zlib.gzip("ab"))).each_char.to_a)

# A member that has been closed has nothing left to say about itself.
closed = Zlib::GzipReader.new StringIO.new(member)
closed.close
begin
  closed.orig_name
rescue Zlib::GzipFile::Error => problem
  p(problem.message)
end
