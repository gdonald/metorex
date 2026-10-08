# The interpreter reads its own flags up to `--` or the program's name, and
# the words after that are the program's, flags and all, as written.
require("rbconfig")

def arguments_of(*words)
  read, write = IO.pipe
  child = spawn(RbConfig.ruby, *words, out: write, err: write)
  write.close
  answer = read.read
  Process.wait(child)
  answer
end

puts(arguments_of("-e", "p ARGV", "--", "-rv", "deep", "a"))
puts(arguments_of("-e", "p ARGV", "x", "-rv"))
puts(arguments_of("-rjson", "-e", "p [ARGV, defined?(JSON)]", "-w", "y", "-Iz"))
puts(arguments_of("-e", "p ARGV", "-r", "json", "--", "k"))
