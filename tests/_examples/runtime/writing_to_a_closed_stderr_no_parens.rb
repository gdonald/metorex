# A child started with its standard error closed finds it closed, and writing
# there raises.
require "rbconfig"
code = 'begin; STDERR.puts("unseen"); rescue SystemCallError => error; puts("rescued #{error.class}"); end'
reader, writer = IO.pipe
pid = Process.spawn RbConfig.ruby, "-e", code, out: writer, err: :close
writer.close
Process.wait pid
p reader.read
