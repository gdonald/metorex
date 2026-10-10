# A socket read of a length waits for that many bytes or for the other end
# to finish, and a read with no length waits for the end. `eof?` waits for
# either and leaves what arrived to be read.
require "socket"

near, far = UNIXSocket.pair
far.write "abc"
waiting = Thread.new { near.read 10 }
Thread.pass until waiting.stop?
p waiting.status
far.write "defghijklm"
p waiting.value
far.close
p near.read
p near.read 5

server = TCPServer.new "127.0.0.1", 0
client = TCPSocket.new "127.0.0.1", server.addr[1]
peer = server.accept
peer.write "12"
p client.eof?
counted = Thread.new { client.read 4 }
Thread.pass until counted.stop?
peer.write "34"
p counted.value
peer.write "line one\r\n\r\nrest"
p client.gets "\r\n\r\n"
peer.close
p client.read
p client.eof?
p client.read 3

left, right = UNIXSocket.pair
left.write "foo"
p left.close_write
p right.read 4
