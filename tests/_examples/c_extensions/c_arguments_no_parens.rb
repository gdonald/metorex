# A C extension reading its arguments with rb_scan_args, reading and building
# Arrays, and building Enumerators with and without a size function.
require "tmpdir"
require_relative "build_helper"

class Counter
  def count_up(from, to)
    from.upto(to) { |number| yield number }
  end
end

directory = Dir.mktmpdir
require build_extension("c_arguments.c", "c_arguments", directory)
arguments = CArguments.new

p arguments.leading_optional 1
p arguments.leading_optional 1, 2
report { arguments.leading_optional }
report { arguments.leading_optional 1, 2, 3 }
p arguments.leading_only 1, 2
report { arguments.leading_only 1 }
p arguments.splat_trailing 1, 2
p arguments.splat_trailing 1, 2, 3, 4
report { arguments.splat_trailing 1 }
p arguments.optional_only
p arguments.optional_only 1, 2
report { arguments.optional_only 1, 2, 3 }
p arguments.splat_only
p arguments.splat_only 1, 2
p arguments.trailing_ignored 1, 2, 3
report { arguments.bad_format 1 }

p arguments.elements [1, "two", :three, nil]
p arguments.same_pointer [1, 2]
p arguments.empty
report { arguments.elements "text" }

walk = arguments.enumerate [1, 2, 3], :each
p walk.class
p walk.to_a
p walk.size
counter = Counter.new
sized = arguments.enumerate_sized counter, :count_up, 2, 4
p sized.to_a
object, given, enumerator = sized.size
p object.equal? counter
p given
p enumerator.equal? sized
p arguments.enumerate_unsized([1, 2], :map).size

FileUtils.rm_rf directory
