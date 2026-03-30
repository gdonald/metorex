# Test that variables, functions, and classes from required files are accessible
require_relative "lib/shared_scope"

# Access variable from required file
puts shared_var

# Call function from required file
puts shared_function()

# Instantiate class from required file
obj = SharedClass.new
puts obj.name
