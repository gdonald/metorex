# Socket.new opens a descriptor straight away, so a socket's settings are
# the operating system's to read and change before it is bound. A value is
# written as a C int when it is a number or a boolean, and as raw bytes
# when it is a String.
require "socket"

socket = Socket.new(:INET, :STREAM)
p(socket.getsockopt(:SOCKET, :TYPE).int == Socket::SOCK_STREAM)
p(socket.setsockopt(:SOCKET, :REUSEADDR, true))
p(socket.getsockopt(:SOCKET, :REUSEADDR).bool)
p(socket.setsockopt(:IP, :TTL, 64))
p(socket.getsockopt(:IP, :TTL).int)
socket.setsockopt(Socket::Option.linger(true, 10))
p(socket.getsockopt(:SOCKET, :LINGER).linger.last)

begin
  socket.setsockopt(:SOCKET, :LINGER, 0)
rescue SystemCallError => error
  p([error.class, error.message])
end
begin
  socket.setsockopt(:SOCKET, :SNDBUF, nil)
rescue TypeError => error
  p(error.message)
end
socket.close
