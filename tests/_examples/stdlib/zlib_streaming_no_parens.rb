# Deflating a little at a time: each call answers what zlib has written so
# far, holding back the bytes a later one could still change, and a block is
# handed the stream in 16384-byte pieces as they fill.
require "zlib"

random = Random.new 0
deflater = Zlib::Deflate.new
first = deflater.deflate random.bytes 20000
second = deflater.deflate random.bytes 20000
last = deflater.finish
p [first.bytesize, second.bytesize, last.bytesize]

pieces = []
streamed = Zlib::Deflate.new
source = Random.new 0
streamed.deflate source.bytes 40000 do |piece|
  pieces << piece.bytesize
end
streamed.finish { |piece| pieces << piece.bytesize }
p pieces

whole = []
generator = Random.new 0
answer = Zlib::Deflate.deflate generator.bytes 40000 do |piece|
  whole << piece
end
again = Random.new 0
expected = again.bytes 40000
read_back = Zlib::Inflate.inflate whole.join
p [answer, whole.map(&:bytesize), read_back == expected]

stopped = Zlib::Deflate.new
stream = Random.new 0
input = stream.bytes 20000
kept = []
stopped.deflate input do |piece|
  kept << piece
  break
end
rest = stopped.finish
restored = Zlib::Inflate.inflate kept.join + rest
p [kept.map(&:bytesize), rest.bytesize, restored == input]

text = (1..20000).map(&:to_s).join ","
packed = Zlib::Deflate.deflate text
p [packed.bytesize, packed.sum]
