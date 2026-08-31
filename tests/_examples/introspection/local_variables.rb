top_level_one = 1
top_level_two = 2
puts local_variables.inspect

def method_locals
  inside_one = 1
  inside_two = 2
  local_variables
end
puts method_locals().inspect

def block_shadows_a_method_local
  shadowed = 1
  1.times do |;shadowed|
    return local_variables
  end
end
puts block_shadows_a_method_local().inspect

def captured_binding
  bound_one = 1
  bound_two = 2
  binding
end
puts eval("local_variables", captured_binding()).inspect

collected = nil
[1].each do
  in_block = 1
  collected = local_variables
end
puts collected.inspect

puts eval("evaluated_one = 1; evaluated_two = 2; local_variables").inspect
