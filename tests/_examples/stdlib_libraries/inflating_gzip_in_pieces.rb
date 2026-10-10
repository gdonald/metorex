# Zlib::Inflate with a window of 32 or more reads gzip as well as zlib, and
# a stream handed over in pieces is read once the whole of it has arrived.
require "zlib"
require "stringio"

packed = StringIO.new
writer = Zlib::GzipWriter.new(packed)
writer.write("maintenance window moves to 02:00")
writer.close
gzipped = packed.string

reader = Zlib::Inflate.new(32 + Zlib::MAX_WBITS)
p(reader.inflate(gzipped.byteslice(0, 12)))
p(reader.inflate(gzipped.byteslice(12, gzipped.bytesize)))

held = +""
answered = Zlib::Inflate.new(16 + Zlib::MAX_WBITS).inflate(gzipped, buffer: held)
p([answered, answered.equal?(held)])

chunks = []
Zlib::Inflate.new(32 + Zlib::MAX_WBITS).inflate(Zlib::Deflate.deflate("zlib wrapped")) { |chunk| chunks << chunk }
p(chunks)

named = StringIO.new
writer = Zlib::GzipWriter.new(named)
writer.orig_name = "notes.txt"
writer.comment = "weekly"
writer.write("named stream")
writer.close
reader = Zlib::Inflate.new(32 + Zlib::MAX_WBITS)
read = named.string.bytes.each_slice(4).map { |piece| reader.inflate(piece.pack("C*")) }
p(read.join)
p(read.first(3))
