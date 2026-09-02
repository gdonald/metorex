require "stringio"

io = StringIO.new("")
io.write("start:")
printf(io, " %s-%d", "value", 7)
io.<<(" shovelled")
io.print(" printed")
io.puts("")
puts(io.string.inspect)

reader = StringIO.new("first\nsecond\n")
puts(reader.gets.inspect)
puts(reader.read.inspect)
puts(reader.read.inspect)
reader.rewind
puts(reader.read(5).inspect)

printf("%s and %s\n", "one", "two")
Kernel.printf("%d\n", 42)

puts(require("stringio").inspect)
puts(Kernel.private_instance_methods.include?(:printf))

path = "/tmp/metorex_file_new_#{Process.pid}.txt"
handle = File.new(path, "w")
handle.write("written")
handle.close
puts(File.read(path))
File.delete(path)
