# A C extension yielding to the block it was called with, working with
# Sets, and making Integers from C integers of every width.
require "tmpdir"
require_relative "build_helper"

class Pair
  def initialize(held)
    @held = held
  end

  def to_ary
    @held
  end
end

directory = Dir.mktmpdir
require(build_extension("c_blocks.c", "c_blocks", directory))
blocks = CBlocks.new

p(blocks.given)
p(blocks.given { 1 })
p(blocks.yield_one(5) { |value| value * 2 })
p(blocks.yield_none { |*values| values })
p(blocks.yield_two(1, 2) { |first, second| first + second })
p(blocks.yield_splat([3, 4]) { |first, second| [second, first] })
report { blocks.yield_splat(3) { 1 } }
p(blocks.yield_splat(Pair.new([5, 6])) { |first, second| first * second })
report { blocks.yield_splat(Pair.new(nil)) { 1 } }
report { blocks.yield_splat(Pair.new(7)) { 1 } }
report { blocks.yield_one(1) }
p(blocks.yield_then { |value| break value + 100 })
p(blocks.yield_then { |value| value })

seen = []
p(blocks.walk(Set[1, 2, 3, 4, 5], seen))
p(seen)
p(blocks.walk(Set[], seen))
p(blocks.set_calls)
p(blocks.empty_set)

p(blocks.conversions)

FileUtils.rm_rf(directory)
