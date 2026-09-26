# `IO.popen` takes the child's environment in a Hash in front, how it is run
# in a Hash behind, and the stream's own options alongside, the way `spawn`
# and `File.open` read them.
metorex = File.expand_path "target/debug/metorex"

p IO.popen({"GREETING" => "hello"}, "echo $GREETING", &:read)
p IO.popen([{"GREETING" => "hi"}, metorex, "-e", "puts ENV['GREETING']"], &:read)
p IO.popen([metorex, "-e", "STDERR.puts :to_err"], err: [:child, :out], &:read)
p IO.popen(["/bin/sh", "-c", "pwd"], chdir: "/", &:read)
p IO.popen([["/bin/sh", "named"], "-c", "echo $0"], &:read)
p IO.popen("echo plain").pid.is_a?(Integer)
p IO.popen("echo positional 1>&2", {err: [:child, :out]}, &:read)

# A command reading its input answers as it reads, while the stream is
# still open for writing.
IO.popen([metorex, "-e", "IO.copy_stream(STDIN, STDOUT)"], "r+") do |both|
  both.write "bar"
  p both.read(3)
end

# The stream is an instance of the class `popen` was called on, and it keeps
# the encodings it was given.
class Stream < IO
end
Stream.popen("echo sub", external_encoding: Encoding::EUC_JP) do |stream|
  p stream.class
  p stream.external_encoding
end

# A stream opened one way refuses the other, and closing it waits for the
# child.
reading = IO.popen "echo foo", "r"
p((reading.write("bar") rescue $!.class))
p reading.read
reading.close
p $?.success?

# The child's error stream may go to a file.
path = "/tmp/metorex_popen_options_#{Process.pid}.txt"
IO.popen([metorex, "-e", "STDERR.print :logged"], err: path, &:read)
p File.read(path)
File.delete path

# A script that is not there is refused as a LoadError.
p IO.popen([metorex, "does_not_exist", {err: [:child, :out]}], &:read)
