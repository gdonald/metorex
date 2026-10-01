# An Addrinfo built from the array IPSocket#addr answers is checked the way
# Ruby checks it: a numeric address pairs STREAM with TCP, DGRAM with UDP,
# and RAW with any protocol, and anything else is put to the resolver.
require "socket"

p Addrinfo.new(["AF_INET", 46102, "localhost", "127.0.0.1"])
p Addrinfo.new(["AF_INET6", 80, "hostname", "::1"])
p Addrinfo.new(["AF_INET", 80, nil, "127.0.0.1"], nil, :STREAM, Socket::IPPROTO_TCP)
raw = Addrinfo.new(["AF_INET", 80, nil, "127.0.0.1"], nil, :RAW, Socket::IPPROTO_ICMP)
p [raw.socktype == Socket::SOCK_RAW, raw.protocol == Socket::IPPROTO_ICMP]
p Addrinfo.new(["AF_INET", 80, nil, "127.0.0.1"], nil, nil, Socket::IPPROTO_UDP).protocol

[
  ["AF_NOPE", 1, nil, "127.0.0.1"],
  ["AF_INET6", 80, "hostname", "127.0.0.1"]
].each do |written|
  begin
    Addrinfo.new(written)
  rescue SocketError => error
    p error.class
  end
end
begin
  Addrinfo.new(["AF_INET", 80, nil, "127.0.0.1"], nil, :DGRAM, Socket::IPPROTO_TCP)
rescue Socket::ResolutionError => error
  p error.class
end

unix = Addrinfo.new(Socket.pack_sockaddr_un("socket"))
p [unix.unix_path, unix.afamily == Socket::AF_UNIX, unix.pfamily, unix.socktype]
