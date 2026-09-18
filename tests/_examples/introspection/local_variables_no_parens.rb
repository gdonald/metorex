top_level_one = 1
top_level_two = 2
puts local_variables.inspect

def method_locals
  inside_one = 1
  inside_two = 2
  local_variables
end
puts method_locals.inspect

def block_shadows_a_method_local
  shadowed = 1
  1.times do |;shadowed|
    return local_variables
  end
end
puts block_shadows_a_method_local.inspect

def captured_binding
  bound_one = 1
  bound_two = 2
  binding
end
puts eval("local_variables", captured_binding).inspect

# A block written in a method sees the method's own locals and nothing from
# outside it.
def block_locals
  [1].each do
    in_block = 1
    return local_variables
  end
end
puts block_locals.inspect

# Code handed to `eval` names its own locals first and the ones it can see
# after them.
puts eval("evaluated_one = 1; evaluated_two = 2; local_variables").inspect
