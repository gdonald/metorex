# Test that circular requires don't cause infinite loops
# circular_a requires circular_b, circular_b requires circular_a
# The second require_relative should be a no-op due to deduplication
require_relative "lib/circular_a"
puts "main done"
