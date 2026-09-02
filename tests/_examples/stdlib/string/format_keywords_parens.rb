puts(format("%s and %s", "a", "b"))
puts(format("test %{name}", name: "value"))
puts(format("%{greeting}, %{who}!", greeting: "hello", who: "world"))
puts(format("%<count>05d", count: 42))
puts(format("%<ratio>.2f", ratio: 3.14159))
puts(Kernel.format("%s", "through the module"))

begin
  format("%{missing}", present: 1)
rescue KeyError => error
  puts(error.message)
end

puts(Kernel.private_instance_methods.include?(:format))
puts(Kernel.private_instance_methods.include?(:sprintf))
