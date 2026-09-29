# Process.exec refuses what cannot start, and otherwise replaces this process
# with the command, run the way its options say.
begin
  Process.exec ""
rescue Errno::ENOENT => error
  p error.class
end
begin
  Process.exec __dir__
rescue Errno::EACCES => error
  p error.class
end
begin
  Process.exec "echo\0"
rescue ArgumentError => error
  p error.message
end
begin
  Process.exec ["/bin/sh"], "-c", "true"
rescue ArgumentError => error
  p error.message
end
$stdout.flush
environment = { "GREETING" => "from the environment" }
Process.exec environment, ["/bin/sh", "named_shell"], "-c", 'echo "$0 $GREETING $(pwd)"', chdir: "/"
puts "not reached"
