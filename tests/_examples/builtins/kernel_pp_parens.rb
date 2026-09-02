pp([1, 2, 3])
pp({a: 1, "b" => 2})
pp("text")

returned = pp(:symbol)
puts(returned.inspect)

pair = pp(1, 2)
puts(pair.inspect)

puts(pp.inspect)
puts(Kernel.private_instance_methods.include?(:pp))
