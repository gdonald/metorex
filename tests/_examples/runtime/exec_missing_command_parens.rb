puts(Kernel.private_instance_methods.include?(:exec))

begin
  exec("definitely_not_a_command_xyz")
rescue Errno::ENOENT => error
  puts(error.message)
end
