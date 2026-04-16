main = TOPLEVEL_BINDING.receiver
puts main.private_methods(true).include?(:ruby2_keywords)
puts main.private_methods.include?(:ruby2_keywords)
