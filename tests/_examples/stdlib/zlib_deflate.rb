require "zlib"

# The bytes zlib itself writes, which a program storing compressed data and
# reading it back elsewhere depends on.
puts Zlib::Deflate.deflate(Array.new(10, 0).pack("C*")).bytes.join(",")
puts Zlib.deflate("1111111111").bytes.join(",")
puts Zlib::Deflate.deflate("\0" * 32 * 1024).bytesize

writer = Zlib::Deflate.new
writer.set_dictionary "aaaaaaaaaa"
writer << "abcdefghij"
puts writer.finish.bytes.join(",")

# A fixed code for a short run, a dynamic code where one pays for itself, and
# a stored block where neither does.
["abcabcabcabc", "The quick brown fox jumps over the lazy dog. " * 50].each do |text|
  packed = Zlib::Deflate.deflate(text)
  puts "#{text.bytesize} -> #{packed.bytesize}"
  puts Zlib.inflate(packed) == text
end

puts Zlib.gzip("12345abcde", level: 6).bytes.last(8).join(",")
puts Zlib.gunzip(Zlib.gzip("round trip")) == "round trip"
