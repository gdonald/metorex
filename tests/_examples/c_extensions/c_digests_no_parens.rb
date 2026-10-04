# A C extension adding a digest algorithm through Digest::Base, and Fiddle
# reading memory at an address.
require "tmpdir"
require "fiddle"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_digests.c", "c_digests", directory)

fold = Digest::Fold.new
p([fold.digest_length, fold.block_length, Digest::Fold.superclass, Digest::MD5.superclass])
fold.update "abcd"
fold << "e"
p fold.hexdigest
p Digest::Fold.hexdigest("abcd")
p Digest::Fold.new("ab").digest.bytes
fold.reset
p fold.digest.bytes
p Digest::MD5.hexdigest("abc")
begin
  Digest::Base.new "abc"
rescue NotImplementedError => error
  p([error.class, error.message])
end

address = CDigests.new.marker_address
pointer = Fiddle::Pointer.new(address)
p([pointer.to_i == address, pointer.size, Fiddle::Pointer[address].to_i == address])
p([pointer[0], pointer[1].chr, pointer[0, 6]])

FileUtils.rm_rf directory
