# A thread runs a step at a time, so a socket that has nothing to give hands
# every waiting thread a turn rather than waiting alone. A server written in
# one thread answers a client written in another.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]

serving = Thread.new do
  client = server.accept
  request = client.gets("\r\n\r\n")
  client.write("heard #{request.chomp.inspect}")
  client.close
end

connection = TCPSocket.new("127.0.0.1", port)
connection.write("ping\r\n\r\n")
p(connection.read(20))
connection.close
serving.join
server.close

# What a read past a separator took but did not hand back is answered before
# anything more is asked of the connection.
pair = TCPServer.new("127.0.0.1", 0)
answering = Thread.new do
  held = pair.accept
  p(held.gets("\r\n"))
  p(held.read(4))
  held.close
end
writing = TCPSocket.new("127.0.0.1", pair.addr[1])
writing.write("head\r\ntail")
answering.join
writing.close
pair.close
