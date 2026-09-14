# Every place writing the same literal shares one frozen string when the
# program was started with --enable-frozen-string-literal, and each place
# writes a mutable string of its own when it was started with the opposite.
held = "shared"
puts "frozen #{held.frozen?}"
puts "shared #{"shared".equal?("shared")}"
