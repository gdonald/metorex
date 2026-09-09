# Command line options read the way the GNU getopt_long function reads them,
# where each option may be written in full or as one letter.
require 'getoptlong'

ARGV.replace ["--size", "10k", "-v", "--check", "a.txt", "b.txt"]

options = GetoptLong.new(
  ["--size", "-s", GetoptLong::REQUIRED_ARGUMENT],
  ["--verbose", "-v", GetoptLong::NO_ARGUMENT],
  ["--check", "--valid", "-c", GetoptLong::NO_ARGUMENT]
)
options.quiet = true

p options.ordering == GetoptLong::PERMUTE
p options.terminated?

found = []
options.each { |name, value| found.push([name, value]) }
p found
p options.terminated?
p ARGV

# A shortened name is taken when only one option starts with it, and a word
# may be written after an equals sign instead of as the next word.
ARGV.replace ["--siz=4k", "-vc"]
short = GetoptLong.new(
  ["--size", "-s", GetoptLong::REQUIRED_ARGUMENT],
  ["--verbose", "-v", GetoptLong::NO_ARGUMENT],
  ["--check", "-c", GetoptLong::NO_ARGUMENT]
)
short.quiet = true
p short.get
p short.get
p short.get
p short.get

# A `--` ends the options, whatever follows it.
ARGV.replace ["-v", "--", "-c"]
stopped = GetoptLong.new(["--verbose", "-v", GetoptLong::NO_ARGUMENT],
                         ["--check", "-c", GetoptLong::NO_ARGUMENT])
stopped.quiet = true
p stopped.get
p stopped.get
p ARGV

# An option that must carry a word and has none raises.
ARGV.replace ["--size"]
missing = GetoptLong.new(["--size", GetoptLong::REQUIRED_ARGUMENT])
missing.quiet = true
begin
  missing.get
rescue GetoptLong::MissingArgument => problem
  p problem.class
end
p missing.error_message
