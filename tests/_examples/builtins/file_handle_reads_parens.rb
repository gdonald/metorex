# An open handle reads a character at a time, moves about, and reports where
# it stands.
path = "/tmp/metorex_handle_parens_reads.txt"
File.write(path, "abc\ndef\n")

handle = File.open(path)
p(handle.getc)
p(handle.pos)
p(handle.readchar)
handle.seek(1, 1)
p(handle.pos)
handle.seek(0, 2)
p(handle.eof?)
begin
  handle.readchar
rescue EOFError => problem
  p problem.message
end
handle.rewind
p(handle.pos)
p(handle.lineno)
handle.lineno = 4
p(handle.lineno)
p(handle.closed?)
handle.close
p(handle.closed?)
handle.reopen(path, "r")
p(handle.closed?)
p(handle.gets)
p(handle.eof?)

# A named pipe is a file both ends open by name.
pipe = "/tmp/metorex_handle_parens_fifo"
File.delete(pipe) if File.exist?(pipe)
File.mkfifo(pipe)
p(File.pipe?(pipe))
p(File.ftype(pipe))
# A second pipe under a name one already holds is refused.
begin
  File.mkfifo(pipe)
rescue Errno::EEXIST => problem
  p(problem.class)
end
File.delete(pipe)

# `lchmod` changes the link rather than what it points at.
target = "/tmp/metorex_handle_parens_target.txt"
link = "/tmp/metorex_handle_parens_link.txt"
File.write(target, "held")
File.delete(link) if File.symlink?(link)
File.symlink(target, link)
# `lchmod` changes the link rather than what it points at, and answers how
# many names it changed. Only some systems allow it at all, so what it
# answers there is left to them.
p(File.lchmod(0755, link).is_a?(Integer))
File.delete(link)
File.delete(target)
File.delete(path)
