# The header `binding.irb` writes shows up to five lines of source on either
# side of the binding, with an arrow at the binding's own line. The line
# numbers are padded to the width of the last index shown, counted from zero.
require "rbconfig"

def header_of(place)
  environment = { "IRBRC" => nil, "HOME" => nil, "XDG_CONFIG_HOME" => nil }
  written = IO.popen([environment, RbConfig.ruby, __FILE__, place], "r+") do |pipe|
    pipe.close_write
    pipe.read
  end
  written.sub(/From: \S+/, "From: here")
end

if ARGV.first == "early"
  binding.irb
elsif ARGV.first == "late"
  binding.irb
else
  puts header_of "early"
  puts header_of "late"
end
