# Test diamond dependency: both B and C require D
# D should only be loaded once
require_relative "lib/diamond_b"
require_relative "lib/diamond_c"
puts "main done"
