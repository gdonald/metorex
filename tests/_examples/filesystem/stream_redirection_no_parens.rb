# A stream points at another place without changing the number the operating
# system holds it under.
require "fcntl"

reader, writer = IO.pipe
p reader.fcntl(Fcntl::F_GETFD) & Fcntl::FD_CLOEXEC
reader.close_on_exec = true
p reader.fcntl(Fcntl::F_GETFD) & Fcntl::FD_CLOEXEC
reader.close
writer.close

first = File.join(Dir.tmpdir, "metorex_stream_first_no_parens.txt")
second = File.join(Dir.tmpdir, "metorex_stream_second_no_parens.txt")
File.write(first, "from the first\n")
File.write(second, "from the second\n")

handle = File.new(first, "r")
p handle.read
handle.reopen(second, "r")
p File.basename(handle.path).sub("_no_parens", "")
p handle.read
handle.close

# `puts` hands its arguments to $stdout, which an Array reaches one line at a
# time.
puts [1, [2, 3]]
puts nil

held = spawn("exit 0")
p held.is_a?(Integer)
Process.wait(held)

File.delete(first)
File.delete(second)
