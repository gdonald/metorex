# ipaddr loads the socket library for the address families it names, so
# requiring it first leaves Socket the class the socket library defines.
require "ipaddr"
require "socket"

p(Socket.class)
p(Socket::AF_INET == IPAddr.new("10.0.0.1").family)
p(IPAddr.new("::1").ipv6?)
p(IPAddr.new("192.168.0.0/16").include?(IPAddr.new("192.168.1.2")))
p(Addrinfo.tcp("127.0.0.1", 80).ip_address)
