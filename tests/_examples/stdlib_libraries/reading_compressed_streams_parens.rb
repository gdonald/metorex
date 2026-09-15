require "zlib"

# A reader takes the compressed text a piece at a time and hands back what it
# stands for once the whole stream has arrived.
deflated = [120, 156, 75, 203, 207, 7, 0, 2, 130, 1, 69].pack("C*")
reader = Zlib::Inflate.new
deflated.each_byte { |byte| reader.<<(byte.chr) }
puts(reader.finish)
reader.close

# Anything after the stream is passed through as it stands.
carried = Zlib::Inflate.new
carried.<<(deflated).<<(nil)
carried.<<("-and more")
puts(carried.finish)
carried.close

# A block is handed the text in 16 KiB pieces.
sizes = []
chunked = Zlib::Inflate.new
chunked.inflate(Zlib::Deflate.deflate("0" * 100_000)) { |piece| sizes.push(piece.length) }
p(sizes)
chunked.close
