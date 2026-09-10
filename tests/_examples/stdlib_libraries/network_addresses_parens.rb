# Network addresses and the connections made to them.

require 'socket'

# An Addrinfo names one endpoint: where it is, what port it sits on, and
# how it is reached.
held = Addrinfo.tcp("127.0.0.1", 80)
p(held.ip_address)
p(held.ip_port)
p(held.ip_unpack)
p(held.afamily == Socket::AF_INET)
p(held.inspect)
p(Addrinfo.tcp("::1", 80).inspect)
p(Addrinfo.udp("127.0.0.1", 80).inspect)
p(Addrinfo.ip("127.0.0.1").inspect)
p(Addrinfo.unix("/tmp/held.sock").inspect)

# An address says what kind it is.
p(Addrinfo.ip("127.0.0.1").ipv4_loopback?)
p(Addrinfo.ip("10.0.0.1").ipv4_private?)
p(Addrinfo.ip("224.0.0.1").ipv4_multicast?)
p(Addrinfo.ip("::1").ipv6_loopback?)
p(Addrinfo.ip("ff02::1").ipv6_mc_linklocal?)
p(Addrinfo.ip("fe80::1").ipv6_linklocal?)
p(Addrinfo.ip("::ffff:127.0.0.1").ipv6_v4mapped?)

# The struct the operating system takes an address in reads back out.
p(Socket.unpack_sockaddr_in Socket.sockaddr_in(80, "127.0.0.1"))
p(Addrinfo.new(held.to_sockaddr).ip_address)

# A connection made to a socket this program is listening on.
server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
client = TCPSocket.new("127.0.0.1", port)
client.write("hello")
accepted = server.accept
p(accepted.read(5))
p(accepted.peeraddr[2])
p(server.addr[0])
accepted.close
client.close
server.close
p(server.closed?)
