puts TOPLEVEL_BINDING.nil?
puts to_s
puts eval("to_s", TOPLEVEL_BINDING)
puts eval "to_s", TOPLEVEL_BINDING
