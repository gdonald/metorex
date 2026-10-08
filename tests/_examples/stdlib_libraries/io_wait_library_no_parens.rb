# io/wait loads, and IO#wait, wait_readable, wait_writable and wait_priority
# answer the stream when it is ready, nil when the time runs out, and the
# events that are ready when asked for a mask of them.
require "socket"
p require("io/wait").class
reader, writer = IO.pipe
p reader.respond_to?(:ready?)
p reader.wait_readable(0.01)
p reader.wait(0.01)
p reader.wait(IO::READABLE, 0.01)
writer.write "x"
p reader.wait_readable(0) == reader
p reader.wait(0.01) == reader
p reader.wait(IO::READABLE, 0.01)
p writer.wait_writable(0) == writer
p writer.wait(IO::WRITABLE, 0)
p writer.wait(0, :write) == writer
p reader.wait(0, :read) == reader
p reader.wait(0, :r) == reader
p writer.wait(0, :w) == writer
p reader.wait(0, :read_write) == reader
p IO::PRIORITY
p reader.wait_priority(0.01)
writer.close
p reader.wait_readable(0) == reader
begin
  reader.wait(-1, :read)
rescue ArgumentError => error
  p error.message
end

server = TCPServer.new "127.0.0.1", 0
client = TCPSocket.new "127.0.0.1", server.addr[1]
accepted = server.accept
p accepted.wait_readable(0.05)
p accepted.wait_priority(0.05)
client.write "x"
client.send "!", Socket::MSG_OOB
sleep 0.05
p accepted.wait_readable(0.05) == accepted
p accepted.wait_priority(0.05) == accepted
p client.wait_writable(0) == client
