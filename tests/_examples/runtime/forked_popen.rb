# `IO.popen("-")` forks. The child answers nil with its standard output and
# input joined to pipes, and the parent answers a stream over the other ends.
io = IO.popen "-"
if io
  p io.gets
  p io.pid.is_a?(Integer)
  io.close
  p $?.success?
else
  puts "hello from child"
  exit!
end

# "r+" joins both ways, and closing the stream waits for the child.
both = IO.popen "-", "r+"
if both
  both.puts "to child"
  both.close_write
  p both.read
  both.close
  p $?.exitstatus
else
  line = STDIN.gets
  puts "child read #{line.inspect}"
  exit! 3
end

# With a block the parent's block gets the stream and answers the result,
# while the child's gets nil and the child exits after it.
answered = IO.popen "-" do |pipe|
  if pipe
    pipe.read
  else
    puts "from the child's block"
  end
end
p answered

# A stream opened one way refuses the other.
writing = IO.popen "-", "w"
if writing
  p((writing.read rescue $!.class))
  writing.close
else
  exit!
end

# `fork` goes through `Process._fork`, and both answer `respond_to?`.
p Process.respond_to?(:fork)
p Process.respond_to?(:_fork)
child = Process._fork
Process.exit! 5 if child == 0
p Process.wait2(child)[1].exitstatus

# A shared buffer over a file is the same bytes in both processes.
path = "/tmp/metorex_forked_popen_#{Process.pid}.txt"
File.write path, "I'm private"
file = File.open path, "r+"
buffer = IO::Buffer.map file
IO.popen "-" do |pipe|
  if pipe
    pipe.read
    p buffer.get_string
  else
    buffer.set_string "I'm shared!"
  end
end
p File.read(path)
file.close
File.delete path

# `exit!` without a status leaves with false, which is 1.
bare = fork { exit! }
p Process.wait2(bare)[1].exitstatus
