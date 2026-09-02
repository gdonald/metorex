begin
  at_exit
rescue ArgumentError => error
  puts(error.message)
end

puts(Kernel.private_instance_methods.include?(:at_exit))

at_exit { puts("registered first") }
at_exit do
  puts("outer")
  at_exit { puts("nested") }
  puts("outer done")
end
at_exit { puts("registered last") }

puts("main body")
