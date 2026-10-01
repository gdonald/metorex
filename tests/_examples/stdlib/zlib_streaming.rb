# Deflating a little at a time: each call answers what zlib has written so
# far, holding back the bytes a later one could still change, and a block is
# handed the stream in 16384-byte pieces as they fill.
require "zlib"

random = Random.new(0)
deflater = Zlib::Deflate.new
first = deflater.deflate(random.bytes(20000))
second = deflater.deflate(random.bytes(20000))
last = deflater.finish
p([first.bytesize, second.bytesize, last.bytesize])

pieces = []
streamed = Zlib::Deflate.new
streamed.deflate(Random.new(0).bytes(40000)) { |piece| pieces << piece.bytesize }
streamed.finish { |piece| pieces << piece.bytesize }
p(pieces)

whole = []
answer = Zlib::Deflate.deflate(Random.new(0).bytes(40000)) { |piece| whole << piece }
p([answer, whole.map(&:bytesize), Zlib::Inflate.inflate(whole.join) == Random.new(0).bytes(40000)])

stopped = Zlib::Deflate.new
input = Random.new(0).bytes(20000)
kept = []
stopped.deflate(input) do |piece|
  kept << piece
  break
end
rest = stopped.finish
p([kept.map(&:bytesize), rest.bytesize, Zlib::Inflate.inflate(kept.join + rest) == input])

text = (1..20000).map(&:to_s).join(",")
p([Zlib::Deflate.deflate(text).bytesize, Zlib::Deflate.deflate(text).sum])
