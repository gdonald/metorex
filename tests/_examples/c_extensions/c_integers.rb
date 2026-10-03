# A C extension writing through RSTRING_PTR, packing Integers into words
# with rb_integer_pack, raising to a power, and defining constants.
require "tmpdir"
require "stringio"
require_relative "build_helper"

class Whole
  def to_int
    258
  end
end

def packed(integers, value, count, size, nails, flags)
  words = ("\0" * (count * size)).b
  sign = integers.pack(value, words, count, size, nails, flags)
  [sign, words.bytes]
end

directory = Dir.mktmpdir
require(build_extension("c_integers.c", "c_integers", directory))
integers = CIntegers.new

text = +"abc"
p(integers.write_byte(text, 1, 0x58))
p(integers.write_byte(text, 0, 0xFF).bytes)
p(text.valid_encoding?)
binary = "\x01\x02".b
integers.write_byte(binary, 1, 0xFE)
p(binary.bytes)
p(integers.write_then_copy(+"xyz", 0x41))
p(integers.pointer_kept(+"same", :upcase!))
p(integers.read_after(+"grow", :succ!))
p(integers.read_after(+"ab", :upcase!))

little = CIntegers::LITTLE_ENDIAN
big = CIntegers::BIG_ENDIAN
two = CIntegers::PACK_2COMP
p(packed(integers, 0, 1, 4, 0, big))
p(packed(integers, 0x0102, 1, 4, 0, big))
p(packed(integers, 0x0102, 1, 4, 0, little))
host = [1].pack("s") == "\1\0" ? little : big
p(packed(integers, 0x0102, 1, 4, 0, CIntegers::NATIVE) == packed(integers, 0x0102, 1, 4, 0, host))
p(packed(integers, 0x01020304, 2, 2, 0, CIntegers::MSWORD | CIntegers::LSBYTE))
p(packed(integers, 0x01020304, 2, 2, 0, CIntegers::LSWORD | CIntegers::MSBYTE))
p(packed(integers, -1, 1, 2, 0, big | two))
p(packed(integers, -1, 1, 2, 0, big))
p(packed(integers, -0x10000, 1, 2, 0, big | two))
p(packed(integers, -0x10000, 1, 2, 0, big))
p(packed(integers, -0x10001, 1, 2, 0, big | two))
p(packed(integers, 0x10000, 1, 2, 0, big))
p(packed(integers, 0x1FF, 2, 1, 4, little))
p(packed(integers, -1, 2, 1, 4, little | two))
p(packed(integers, 2.9, 1, 1, 0, big))
p(packed(integers, Whole.new, 1, 2, 0, big | CIntegers::GENERIC))
report { packed(integers, 1, 1, 1, 0, big | CIntegers::FORCE_BIGNUM) }
report { packed(integers, 1, 2, 1, 0, CIntegers::MSBYTE) }
report { packed(integers, 1, 1, 1, 0, CIntegers::MSWORD | CIntegers::LSWORD | CIntegers::MSBYTE) }
report { packed(integers, 1, 1, 1, 0, CIntegers::MSWORD) }
report { packed(integers, 1, 1, 1, 0, CIntegers::MSBYTE | CIntegers::LSBYTE) }
report { packed(integers, 1, 1, 0, 0, big) }
report { packed(integers, 1, 1, 1, 8, big) }
report { packed(integers, "1", 1, 1, 0, big) }

p(integers.power(2, 10))
p(integers.power(-3, 3))
p(integers.power(7, 30))

p(CIntegers::BIG_ENDIAN)
module Holder
end
integers.define_const(Holder, "Named", 1)
p(Holder::Named)
$VERBOSE = false
$stderr = StringIO.new
integers.define_const(Holder, "lower", 2)
warned = $stderr.string
$stderr = STDERR
puts(warned.sub(/\A.*\//, "").sub(File.basename(__FILE__), "example.rb"))
$VERBOSE = nil
integers.define_const(Holder, "quiet", 3)
Holder.freeze
report { integers.define_const(Holder, "Frozen", 4) }
report { integers.define_const(Holder, "frozen", 4) }

FileUtils.rm_rf(directory)
