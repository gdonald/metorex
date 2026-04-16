# Test that require_relative auto-detects .rb extension
# Requiring "lib/helper" should find "lib/helper.rb"
require_relative "../lib/helper"
puts helper_var
puts helper_method()
