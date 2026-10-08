# OptionParser reads options in order, handing each argument that is not an
# option to the block or stopping at it, or permuted, collecting them. A short
# option's value may follow it or be written attached, and short flags may be
# written together.
require "optparse"

parser = OptionParser.new
parser.on("-p") { |value| p [:p, value] }
parser.on("-m ") { |value| p [:m, value] }
parser.on("-n VALUE") { |value| p [:n, value] }
parser.on("-r") { |value| p [:r, value] }
parser.on("--port=PORT") { |value| p [:port, value] }
parser.on("--[no-]color") { |value| p [:color, value] }

seen = []
words = ["-p", "x", "-m", "644", "-n5", "-pr", "-rn", "7", "--port", "80", "--port=81", "--no-color", "y"]
parser.order!(words) { |word| seen << word }
p seen, words
p parser.order!(["-p", "x", "-p"])
p parser.permute!(["x", "-p", "y", "--", "-m", "z"])
p parser.parse(["a", "-p", "b"])
kept = ["-p", "--", "-m"]
handed = []
parser.order!(kept) { |word| handed << word }
p handed, kept
[["-z"], ["-n"], ["--nope"]].each do |words_given|
  parser.order! words_given
rescue OptionParser::ParseError => error
  p [error.class, error.message]
end
