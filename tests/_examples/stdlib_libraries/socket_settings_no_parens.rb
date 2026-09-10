require 'socket'

# One setting on a socket names the level it belongs to and the value it was
# given, whichever way the level and the option are written.
keepalive = Socket::Option.bool(:INET, :SOCKET, :KEEPALIVE, true)
p [keepalive.family == Socket::AF_INET, keepalive.level == Socket::SOL_SOCKET]
p keepalive.optname == Socket::SO_KEEPALIVE
p keepalive.bool
p keepalive.data == [1].pack("i")

ttl = Socket::Option.int(:INET, :IP, :TTL, 4)
p ttl.int

p Socket::Option.linger(nil, 0).inspect
p Socket::Option.linger(true, 30).inspect
p Socket::Option.linger(true, 30).linger

begin
  Socket::Option.linger(1, 4).bool
rescue TypeError => problem
  p(problem.class)
end
begin
  Socket::Option.new(:INET4, :SOCKET, :KEEPALIVE, [0].pack("i"))
rescue SocketError => problem
  p(problem.message)
end

# A packed address reads back as the port and the address it holds.
p Socket.unpack_sockaddr_in(Socket.pack_sockaddr_in(80, "127.0.0.1"))
p Socket.unpack_sockaddr_in(Socket.pack_sockaddr_in("http", "127.0.0.1"))
p Socket.unpack_sockaddr_in(Socket.pack_sockaddr_in(80, Socket::INADDR_ANY))
p Socket.pack_sockaddr_in(80, "127.0.0.1").bytesize
p Addrinfo.new(Socket.pack_sockaddr_in(80, "127.0.0.1")).pfamily == Socket::PF_UNSPEC
p Addrinfo.unix("/foo", Socket::SOCK_DGRAM).inspect

# Two ends joined to each other carry no name of their own.
first, second = UNIXSocket.socketpair
p first.inspect == "#<UNIXSocket:fd #{first.fileno}>"
p first.path
p [first.binmode?, first.nonblock?, first.close_on_exec?]
first.close
second.close
