# Test that require_relative works with explicit .rb extension
require_relative "../lib/helper.rb"
puts HELPER_VALUE
puts helper_method()
