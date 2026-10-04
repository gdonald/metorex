# A C extension building Arrays, reading and writing their elements, and
# calling a method with a block that runs a C function.
require "tmpdir"
require_relative "build_helper"

class Listing
  def to_ary
    ["converted"]
  end
end

class Letters
  def each
    yield "a"
    yield "b"
  end
end

directory = Dir.mktmpdir
require(build_extension("c_arrays.c", "c_arrays", directory))
arrays = CArrays.new

p(arrays.convert([1, 2]))
p(arrays.convert({ "key" => "value" }))
p(arrays.convert("text"))
p(arrays.capacity(5))
report { arrays.capacity(-1) }
p(arrays.triple(1, 2, 3))
p(arrays.triple_args(:a, :b, :c))
p(arrays.element([1, 2, 3], 1))
p(arrays.set_element([1, 2, 3], 2, 42))

filled = [1, 2, 3]
p(arrays.fill(filled, :set).equal?(filled))
p(filled)
source = [7, 8, 9]
target = [0, 0, 0]
arrays.copy_into(source, target)
p([source, target])

p(arrays.aref([1, 2, 3, 4], 1))
p(arrays.aref([1, 2, 3, 4], 1, 2))
p(arrays.aref([1, 2, 3, 4], 1..))
p(arrays.cat([1], 2, 3))
report { arrays.cat([].freeze, 1) }
p(arrays.clear([1, 2]))
p(arrays.concat([1], [2, 3]))
deleting = [1, 2, 3, 2]
p(arrays.delete_element(deleting, 2))
p(arrays.delete_element(deleting, 5))
p(deleting)
p(arrays.delete_at([1, 2, 3], -1))
p(arrays.delete_at([1, 2, 3], 5))
p(arrays.freeze([1]).frozen?)
p(arrays.includes([1, 2], 2))
p(arrays.includes([1, 2], 3))
p(arrays.join([1, 2, 3], "-"))
p(arrays.plus([1], [2]))
reversed = [1, 2, 3]
p(arrays.reverse(reversed).equal?(reversed))
p(reversed)
p(arrays.rotate([1, 2, 3, 4], 1))
p(arrays.rotate([1, 2, 3, 4], -1))
report { arrays.rotate([].freeze, 1) }
shifted = [1, 2]
p(arrays.shift(shifted))
p(shifted)
p(arrays.shift([]))
unsorted = [3, 1, 2]
p(arrays.sort(unsorted))
p(unsorted)
p(arrays.sort_bang(unsorted))
p(unsorted)

p(arrays.subseq([1, 2, 3, 4, 5], 1, 3))
p(arrays.subseq([1, 2, 3, 4, 5], 4, 3))
p(arrays.subseq([1, 2, 3, 4, 5], 5, 1))
p(arrays.subseq([1, 2, 3, 4, 5], 6, 1))
p(arrays.subseq([1, 2, 3, 4, 5], -1, 1))
p(arrays.subseq([1, 2, 3, 4, 5], 1, -1))

same = [1]
p(arrays.to_ary(same).equal?(same))
p(arrays.to_ary(Listing.new))
p(arrays.to_ary(5))
p(arrays.to_s([1, "two", :three]))
p(arrays.pair(:key, [1, 2]))
p(arrays.cleared(1, 2))

p(arrays.collect_each([1, 2, 3], "each"))
p(arrays.collect_each({ a: 1, b: 2 }, "each_pair"))
yielded = []
arrays.yield_each(Letters.new) { |letter| yielded << letter.upcase }
p(yielded)
report { arrays.yield_each(Letters.new) }
p(arrays.each_slice([1, 2, 3, 4, 5], 2))

FileUtils.rm_rf(directory)
