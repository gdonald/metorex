metorex = "target/debug/metorex"

output = IO.popen([metorex, "-n", "-e", 'print "got: ", $_'], "r+") do |io|
  io.puts "a line"
  io.close_write
  io.read
end
puts output

second = IO.popen([metorex, "-e", 'print "no input"'], "r+") do |io|
  io.close_write
  io.read
end
puts second
puts $?.exitstatus
