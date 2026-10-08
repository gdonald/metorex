require "rbconfig"

interpreter = RbConfig.ruby

output = IO.popen([interpreter, "-n", "-e", 'print "got: ", $_'], "r+") do |io|
  io.puts "a line"
  io.close_write
  io.read
end
puts output

second = IO.popen([interpreter, "-e", 'print "no input"'], "r+") do |io|
  io.close_write
  io.read
end
puts second
puts $?.exitstatus
