# A literal written where a test goes earns a warning while the code is
# read, whatever the verbosity: a regexp or a string literal, and an
# integer at either end of a flip-flop. A program run from a file, code
# given to eval, and code given to RubyVM::AbstractSyntaxTree.parse each
# name where the literal was written.
require "rbconfig"
require "stringio"

fixture = File.join(__dir__, "literal_conditions_fixture.rb")
output = IO.popen([RbConfig.ruby, fixture], err: [:child, :out], &:read)
puts output.gsub(File.dirname(fixture) + "/", "")

captured = StringIO.new
$stderr = captured
eval "if /a/ then end", binding, "evaluated.rb", 7
RubyVM::AbstractSyntaxTree.parse "x = 1\nif (1..2) then end\nwhile /b/ do end"
$stderr = STDERR
puts captured.string

quiet = IO.popen([RbConfig.ruby, "-W0", fixture], err: [:child, :out], &:read)
puts quiet
