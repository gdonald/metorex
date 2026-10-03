# A C extension making Regexps, matching with them, reading matches and
# $~, comparing bytes without case, and converting values with StringValue.
require "tmpdir"
require_relative "build_helper"

class Text
  def to_str
    "converted"
  end
end

directory = Dir.mktmpdir
require build_extension("c_regexps.c", "c_regexps", directory)
regexps = CRegexps.new

made = regexps.make "b+", Regexp::IGNORECASE
p made
p "aBBc" =~ made
p regexps.make("\xFF".b, 0).source.bytes
p regexps.compile "x(y)"
p regexps.options(/z/mx)

p regexps.match(/é(.)/, "aaéb")
p regexps.last[1]
p regexps.match(/q/, "aaa")
p regexps.last

found = /(a)(b)?(c)/.match "ac"
p [regexps.nth(1, found), regexps.nth(2, found), regexps.nth(3, found), regexps.nth(4, found)]
p [regexps.nth(-1, found), regexps.nth(-3, found), regexps.nth(-4, found), regexps.nth(1, nil)]

regexps.set_last(/k/.match "k")
p $~[0]
regexps.set_last nil
p $~

p [regexps.compare("Hello", "HELLO", 5), regexps.compare("abc", "ABD", 3), regexps.compare("x", "y", 0)]

p regexps.string_value Text.new
report { regexps.string_value 5 }
p regexps.string_pointer "A"
p regexps.c_string_length "four"
report { regexps.c_string_length "a\0b" }
report { regexps.c_string_length "ab\0".encode("UTF-16LE") }
p regexps.c_string_length "ab".encode("UTF-16LE")

FileUtils.rm_rf directory
