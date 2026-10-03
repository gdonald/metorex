# A C extension converting Integers too wide for a fixnum to and from C
# numbers and strings, comparing them, packing them into longs, and reading
# any number as a C double.
require "tmpdir"
require "stringio"
require_relative "build_helper"

class Measured
  def to_f
    2.5
  end
end

class Wrong
  def to_f
    "no"
  end
end

directory = Dir.mktmpdir
require(build_extension("c_bignums.c", "c_bignums", directory))
bignums = CBignums.new
long_max = 2**63 - 1
long_min = -2**63

p([bignums.to_long(long_max), bignums.to_long(long_min), bignums.to_long(5)])
report { bignums.to_long(long_max + 1) }
report { bignums.to_long(long_min - 1) }
report { bignums.to_long(2**70) }
p(bignums.to_long_long(long_min))
report { bignums.to_long_long(long_max + 1) }
report { bignums.to_long_long(2**70) }
p([bignums.to_unsigned_long(2**64 - 1), bignums.to_unsigned_long(-1), bignums.to_unsigned_long(long_min)])
report { bignums.to_unsigned_long(long_min - 1) }
report { bignums.to_unsigned_long(2**64) }

p(bignums.to_double(2**70))
$stderr = StringIO.new
$VERBOSE = true
p(bignums.to_double(-(10**400)))
warned = $stderr.string
$stderr = STDERR
$VERBOSE = false
puts(warned.sub(/\A.*warning: /, "warning: "))
p(bignums.from_double(219238102380912830988.5))
p(bignums.from_double(-2.75))
report { bignums.from_double(Float::INFINITY) }
report { bignums.from_double(-Float::INFINITY) }
report { bignums.from_double(Float::NAN) }

p(bignums.to_text(2**70, 10))
p(bignums.to_text(2**70, 16))
p([bignums.sign(2**70), bignums.sign(-(2**70)), bignums.sign(0)])
p([bignums.compare(2**70, 2**71), bignums.compare(2**70, 5), bignums.compare(2**70, 2**70)])

p(bignums.pack(2**64 - 1, 2))
p(bignums.pack(-(2**71 + 1), 2))
p(bignums.pack(5, 1))
p([bignums.size(2**71), bignums.size(255), bignums.size(-256), bignums.size(0)])

p([bignums.to_c_double(3), bignums.to_c_double(2**70), bignums.to_c_double(1.5), bignums.to_c_double(Rational(1, 4))])
p(bignums.to_c_double(Measured.new))
report { bignums.to_c_double(nil) }
report { bignums.to_c_double(true) }
report { bignums.to_c_double(:sym) }
report { bignums.to_c_double("1.5") }
report { bignums.to_c_double(Object.new) }
report { bignums.to_c_double(Wrong.new) }
p(bignums.empty)

FileUtils.rm_rf(directory)
