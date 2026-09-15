require 'socket'

# Two sockets already joined to each other, in either family.
first, second = Socket.pair(Socket::AF_UNIX, Socket::SOCK_STREAM)
first.puts('over a path')
p(second.gets)
p(first.local_address.afamily == Socket::AF_UNIX)
first.close
second.close

# A socket carrying each message on its own says where each one came from.
sending = Socket.new(:INET, :DGRAM)
receiving = Socket.new(:INET, :DGRAM)
receiving.bind(Socket.sockaddr_in(0, '127.0.0.1'))
sending.send('one message', 0, receiving.getsockname)
message, from = receiving.recvfrom(11)
p(message)
p(from.class)
p(from.ip_address)
sending.close
receiving.close

# Reading into a buffer keeps the encoding the buffer was tagged with.
listener = Socket.new(:INET, :STREAM)
listener.bind(Socket.sockaddr_in(0, '127.0.0.1'))
listener.listen(1)
client = Socket.tcp('127.0.0.1', listener.connect_address.ip_port)
client.write('bytes')
accepted, address = listener.accept
buffer = 'held'.dup.force_encoding(Encoding::UTF_8)
accepted.read(5, buffer)
p(buffer)
p(buffer.encoding)
p(address.class)
accepted.close
client.close
listener.close

# What the system knows about names, ports, and addresses.
p(Socket.getservbyname('ftp'))
p(Socket.gethostbyname('<broadcast>')[0])
p(Socket.getnameinfo(['AF_INET', 21, '127.0.0.1'], Socket::NI_NUMERICHOST | Socket::NI_NUMERICSERV))
