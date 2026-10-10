# A C extension writing to IO objects, asking about the descriptors they
# hold, and waiting until a descriptor is ready.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_io.c", "c_io", directory)
io = CIO.new
path = File.join(directory, "written.txt")

reader, writer = IO.pipe
both = File.open(path, "w+")
p([io.descriptor(reader) == reader.fileno, io.mode(reader) & 3, io.mode(writer) & 3, io.mode(both) & 3])
writer.sync = true
both.binmode
p([io.mode(writer) & 8, io.mode(both) & 4])
p io.struct_fields(both) == [both.fileno, io.mode(both), path]
p([io.check_readable(reader), io.check_writable(writer), io.check_readable(both), io.check_writable(both)])
report { io.check_readable writer }
report { io.check_writable reader }
p io.set_nonblock reader
p io.fix_cloexec writer
p io.cloexec_open path
duplicate, number = io.cloexec_dup(reader, nil)
p([duplicate, number > 2])
duplicate, number = io.cloexec_dup(reader, 50)
p([duplicate, number >= 50])
p([io.path(both) == path, io.closed(both), io.check_io(both).equal?(both), io.check_io({})])
p io.taint_check(both).equal?(both)

p io.addstr(both, :symbol).equal?(both)
io.io_printf(both, ["-%s-%d", "a", 1])
io.io_print(both, ["b", 2])
io.io_puts(both, ["c", 3])
p io.io_write both, "end"
p io.binmode(both).equal?(both)
both.rewind
p both.read

p io.io_wait writer, IO::WRITABLE, nil
p io.io_wait reader, IO::READABLE, 0
p io.io_wait reader, IO::READABLE, 0.01
p io.maybe_wait Errno::EINTR::Errno, writer, IO::WRITABLE, nil
p io.maybe_wait 0, writer, IO::WRITABLE, nil
p io.maybe_wait Errno::EAGAIN::Errno, writer, IO::WRITABLE, nil
p([io.maybe_wait_readable(0, reader, nil), io.maybe_wait_writable(Errno::EINTR::Errno, writer, nil)])
report { io.maybe_wait_readable Errno::EAGAIN::Errno, reader, 0 }
writer.write("ready")
p([io.maybe_wait_readable(Errno::EAGAIN::Errno, reader, nil), io.wait_fd(reader), io.fd_writable(writer)])
p io.fd_select([reader], [writer], [], nil)
p io.fd_select([reader], [], [], 1)
reader.read_nonblock(5)
p io.fd_select([reader], [], [], 0)
late_reader, late_writer = IO.pipe
filler = Thread.new { sleep 0.05; late_writer.write("late") }
p io.wait_fd late_reader
filler.join
late_writer.close
p io.io_wait late_reader, IO::READABLE | IO::WRITABLE, 0

report { io.descriptor IO.allocate }
report { io.maybe_wait 0, IO.allocate, IO::READABLE, nil }
both.close
report { io.descriptor both }
p([io.struct_fields(both).first(2), io.struct_fields(both).last == path, io.closed(both)])
report { io.check_readable both }
frozen = File.open(path)
frozen.freeze
begin
  io.taint_check(frozen)
rescue FrozenError => error
  p(error.class)
end
frozen.close
p frozen.closed?

opened = io.open_descriptor(IO, reader.fileno, 2, "piped.txt", 60, "US-ASCII", "UTF-8")
p([opened.class, opened.fileno == reader.fileno, opened.path, opened.timeout, opened.internal_encoding, opened.external_encoding])
p io.open_descriptor(File, writer.fileno, 2 | 4, "named", nil, nil, nil).class
named = Object.new
def named.to_str = "converted.txt"
p io.open_descriptor(IO, reader.fileno, 3, named, nil, nil, nil).path

closing_reader, closing_writer = IO.pipe
p [io.io_close(closing_writer), closing_writer.closed?]
waiting_reader, _waiting_writer = IO.pipe
waiter = Thread.new { io.wait_fd waiting_reader }
sleep 0.05
waiter.kill
p waiter.join.status
read_write = File.open path, "r+"
p io.open_descriptor(File, read_write.fileno, 3, "both", nil, nil, nil).class
read_write.close
wrong_way = io.open_descriptor(File, writer.fileno, 1, "wrong.txt", nil, nil, nil)
begin
  wrong_way.read_nonblock 1
rescue Errno::EBADF => error
  p error.class
end
FileUtils.rm_rf directory
