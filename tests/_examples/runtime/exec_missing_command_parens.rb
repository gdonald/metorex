puts(Kernel.private_instance_methods.include?(:exec))

begin
  exec("definitely_not_a_command_xyz")
rescue Errno::ENOENT => error
  puts(error.message)
end

# A command that never started leaves a broken pipe raising, as before.
reader, writer = IO.pipe
reader.close
begin
  writer.write("lost")
rescue Errno::EPIPE => error
  puts(error.class)
end
