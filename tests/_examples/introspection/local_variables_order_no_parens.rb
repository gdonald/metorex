def declared_in_order
  third = 3
  first = 1
  second = 2
  binding.local_variables
end
puts declared_in_order.inspect

def own_locals_come_before_enclosing_ones
  outer_one = 1
  outer_two = 2
  proc { inner_one = 3; binding.local_variables }.call
end
puts own_locals_come_before_enclosing_ones.inspect

def a_local_shadowing_a_builtin_is_still_a_local
  p = proc { :shadowing }
  binding.local_variables
end
puts a_local_shadowing_a_builtin_is_still_a_local.inspect
