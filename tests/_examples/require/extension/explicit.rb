# Test that require_relative works with explicit .rb extension
require_relative "../lib/helper.rb"
puts helper_var
puts helper_method()
