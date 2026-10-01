# A connect told not to wait starts the connection and reports that it is
# under way. Once the socket can be written to, a second try finds it
# connected, which Linux reports by answering 0 and macOS by raising
# Errno::EISCONN. Without exceptions both answer 0.
require "socket"

server = TCPServer.new("127.0.0.1", 0)
address = Socket.sockaddr_in(server.addr[1], "127.0.0.1")
socket = Socket.new(:INET, :STREAM)
begin
  socket.connect_nonblock address
rescue IO::WaitWritable => error
  p [error.class, error.message]
end
IO.select(nil, [socket])
p socket.connect_nonblock(address, exception: false)
peer = server.accept
socket.write "connected"
p peer.recv(9)

waiting = Socket.new(:INET, :STREAM)
p waiting.connect_nonblock(address, exception: false)
p Socket.new(:INET, :STREAM).connect_address rescue p $!.message
