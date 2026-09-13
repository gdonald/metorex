# Constants, methods, and classes from a required file are reachable, and its
# local variables are not: each file keeps its own top-level locals.
require_relative "lib/shared_scope"

puts SHARED_VALUE

puts shared_function()

obj = SharedClass.new
puts obj.name

begin
  puts shared_var
rescue NameError => refused
  puts refused.message
end
