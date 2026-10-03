# A C extension formatting text the way rb_sprintf, rb_raise and rb_warn do,
# with PRIsVALUE writing a VALUE's to_s, or its inspect with the + flag.
require "tmpdir"
require "stringio"
require_relative "build_helper"

class Odd
  def to_s
    5
  end
end

directory = Dir.mktmpdir
require build_extension("c_formats.c", "c_formats", directory)
formats = CFormats.new

p formats.numbers
p formats.floats
p formats.texts "café"
p formats.texts("café").encoding
p formats.texts :sym
p formats.numbers.encoding
p formats.pointer_text
p formats.unknown
p formats.appended(+"one")
report { formats.raised ArgumentError, [1, 2] }
report { formats.raised KeyError, nil }

$stderr = StringIO.new
$VERBOSE = false
formats.warned "this"
$VERBOSE = true
formats.warned "that"
$VERBOSE = nil
formats.warned "nothing"
warned = $stderr.string
$stderr = STDERR
puts warned.gsub(/^.*\//, "").gsub(File.basename(__FILE__), "example.rb")
$VERBOSE = false

p formats.bytes
p formats.bytes.map(&:encoding).uniq
p formats.as_strings 12
p formats.as_strings "text"
p formats.as_strings(Odd.new)[0].start_with? "#<Odd"

FileUtils.rm_rf directory
