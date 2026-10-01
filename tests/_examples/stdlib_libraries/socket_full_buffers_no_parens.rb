# A write on a socket whose buffer is full waits for the other end to read,
# while a send told not to wait writes what fits and refuses only when
# nothing does.
require "socket"

reader, writer = UNIXSocket.pair
begin
  10.times { writer.sendmsg_nonblock("x" * 1_000_000) }
rescue IO::WaitWritable => error
  p [error.class, error.message]
end
p writer.sendmsg_nonblock("x" * 1_000_000, exception: false)
reader.close
writer.close

server = TCPServer.new("127.0.0.1", 0)
client = TCPSocket.new("127.0.0.1", server.addr[1])
peer = server.accept
counted = Thread.new do
  total = 0
  while (chunk = peer.read(65_536)) && !chunk.empty?
    total += chunk.bytesize
  end
  total
end
p client.write("y" * 3_000_000)
client.close
p counted.value
peer.close
server.close
