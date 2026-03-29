# Test file: test_strings.rb
# Discovered by --test flag (matches test_*.rb pattern)

assert_equal(5, "hello".length)
assert_equal("HELLO", "hello".upcase)
assert_equal("hello", "HELLO".downcase)
puts "test_strings passed"
