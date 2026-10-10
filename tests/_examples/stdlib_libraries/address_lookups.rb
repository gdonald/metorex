# A lookup answers the addresses in the order the resolver gives them, one
# for each kind of socket the resolver pairs with each address, and reads an
# empty host as the address that means every one.
require "socket"

ordered = Socket.getaddrinfo("localhost", 80, nil, :STREAM).map { |entry| entry[3] }
p(Addrinfo.getaddrinfo("localhost", 80, nil, :STREAM).map(&:ip_address) == ordered)
p(Addrinfo.tcp("localhost", 80).ip_address == ordered.first)
p(Addrinfo.getaddrinfo(nil, 80, nil, :STREAM).map(&:ip_address) ==
  Socket.getaddrinfo(nil, 80, nil, :STREAM).map { |entry| entry[3] })

begin
  Addrinfo.getaddrinfo("", 80, :INET6, :STREAM)
rescue Socket::ResolutionError => error
  p(error.class)
end
p(Addrinfo.getaddrinfo("", 80, :INET, :STREAM))
p(Addrinfo.getaddrinfo("<any>", 80, :INET, :STREAM))
p(Addrinfo.getaddrinfo(nil, 80, :INET, :STREAM, nil, Socket::AI_PASSIVE))

p(Addrinfo.getaddrinfo("127.0.0.1", 80))
p(Addrinfo.getaddrinfo("::1", nil, nil, :DGRAM))
p(Addrinfo.tcp("127.0.0.1", "http"))
p(Addrinfo.getaddrinfo("localhost", "http", :INET, :STREAM).map(&:inspect).uniq)

p(Addrinfo.new(Socket.sockaddr_in(80, "127.0.0.1"), :INET, :RAW, 0))
p(Addrinfo.new(Socket.sockaddr_in(80, "127.0.0.1"), :INET, :STREAM, Socket::IPPROTO_UDP))
p(Addrinfo.new(Socket.sockaddr_in(80, "127.0.0.1"), :INET, :RAW, Socket::IPPROTO_TCP))
