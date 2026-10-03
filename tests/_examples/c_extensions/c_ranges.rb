# A C extension making Ranges, reading their ends, turning ranges and
# arithmetic sequences into a start and a length, and storing into Arrays.
require "tmpdir"
require_relative "build_helper"

class Span
  def begin
    2
  end

  def end
    6
  end

  def exclude_end?
    true
  end
end

class HalfSpan
  def begin
    2
  end

  def end
    6
  end
end

directory = Dir.mktmpdir
require(build_extension("c_ranges.c", "c_ranges", directory))
ranges = CRanges.new

p(ranges.make(1, 4, false))
p(ranges.make("a", "c", true).to_a)
p(ranges.make(1, 4, false).frozen?)
report { ranges.make(1, "a", false) }

p(ranges.ends(3...9))
p(ranges.ends(Span.new))
p(ranges.ends(HalfSpan.new))
p(ranges.ends(1.step(10, 3)))

p(ranges.start_length(2..5, 10, 0))
p(ranges.start_length(2...5, 10, 0))
p(ranges.start_length(-3.., 10, 0))
p(ranges.start_length(..-2, 10, 0))
p(ranges.start_length(8..20, 10, 0))
p(ranges.start_length(12..20, 10, 0))
p(ranges.start_length(12..20, 10, 1))
report { ranges.start_length(12..20, 10, 2) }
report { ranges.start_length(-20..2, 10, 1) }
p(ranges.start_length(Span.new, 10, 0))
p(ranges.start_length(5, 10, 0))

p(ranges.parts(1.step(10, 3)))
p(ranges.parts((1...10) % 2))
p(ranges.parts(3..7))
p(ranges.parts(Span.new))
p(ranges.parts(Object.new))

p(ranges.start_length_step((1..9).step(2), 10, 0))
report { ranges.start_length_step((1..12).step(2), 10, 0) }
report { ranges.start_length_step((-20..2).step(2), 10, 0) }
p(ranges.start_length_step((8..2).step(-2), 10, 0))
p(ranges.start_length_step(((2...8) % -1), 10, 1))
p(ranges.start_length_step((12..20).step(1), 10, 0))
p(ranges.start_length_step(Object.new, 10, 0))

p(ranges.store([1, 2], 4, :later))
p(ranges.store([1, 2], -1, :last))
report { ranges.store([1, 2], -3, :before) }

FileUtils.rm_rf(directory)
