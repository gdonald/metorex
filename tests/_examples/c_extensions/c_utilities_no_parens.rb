# A C extension reading keywords and its block through rb_scan_args, taking
# keywords out of a Hash with rb_get_kwargs, breaking out of the block that
# called it, naming where it was called from, and reading numbers with
# ruby_strtod.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_utilities.c", "c_utilities", directory)
utilities = CUtilities.new
action = -> { :acted }

p utilities.scanned(1, size: 2)
report { utilities.scanned 1, { size: 2 } }
p utilities.scanned(size: 2).first(3)
used = utilities.scanned(&action)
p [used[0], used[1], used[2], used[3].equal?(action), used[4]]
report { utilities.scanned 1, 2 }
p utilities.scanned_last_hash({ size: 2 })
p utilities.scanned_last_hash 3
report { utilities.block_proc }
p utilities.block_proc(&action).equal? action

held = { a: 1, b: 2, c: 3 }
p utilities.keywords(held, [:b, :a], 2, -1)
p held
p utilities.keywords({ a: 1 }, [:a, :z], 1, 1)
p utilities.keywords(nil, [:a, :b], 0, 2)[0]
report { utilities.keywords({}, [:a, :b], 2, 0) }
report { utilities.keywords({ a: 1, x: 2, y: 3 }, [:a], 1, 0) }
report { utilities.keywords({ a: 1, x: 2 }, [:a], 1, 0) }
p utilities.keywords_present({ a: 1 }, :a)
report { utilities.keywords_present({ b: 1 }, :a) }

p([1, 2, 3].each { |number| utilities.stop :stopped if number == 2 })
p([1, 2, 3].map { |number| number == 2 ? utilities.stop_plain : number })
outcome = [1, 2].map do |outer|
  [10, 20].each { |inner| utilities.stop_plain if inner == 20 }
  outer
end
p outcome

file, line = utilities.where
p [File.basename(file).sub("_no_parens", ""), line == __LINE__ - 1]
p utilities.narrowed(2**31 - 1)
report { utilities.narrowed 2**31 }
report { utilities.narrowed(-2**31 - 1) }

["14.25test", "  -3e2x", "+", "test", "1e", "0.", "000", ".5", "0x1Ap1rest", "0x.8", "0x", "0xg", "1e400", "0x0", "-0x10p-1"].each do |text|
  p [text, utilities.parsed(text)]
end

source = { kept: 1 }
copy = utilities.copied source
p [copy, copy.equal?(source)]

FileUtils.rm_rf directory
