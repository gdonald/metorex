require 'socket'
require 'tmpdir'

# The same as `streams_over_a_path.rb`, written without parentheses.
# A socket named by a path hands a stream of this program to the other end,
# which takes it up as a stream of its own.
holding = File.join(Dir.tmpdir, "metorex_send_io_#{Process.pid}.txt")
File.write(holding, 'read through the handed stream')
opened = File.open(holding)

path = File.join(Dir.tmpdir, "metorex_pass_#{Process.pid}.sock")
File.delete(path) if File.exist?(path)
listening = UNIXServer.new(path)
client = UNIXSocket.new(path)
client.send_io opened

accepted = listening.accept
taken = accepted.recv_io
p taken.read

taken.close
accepted.close
client.close
opened.close
listening.close
File.delete(path) if File.exist?(path)
File.delete(holding) if File.exist?(holding)

# A socket named by a path also carries each message on its own, saying
# which path each one came from.
first = File.join(Dir.tmpdir, "metorex_dgram_a_#{Process.pid}.sock")
second = File.join(Dir.tmpdir, "metorex_dgram_b_#{Process.pid}.sock")
[first, second].each { |name| File.delete(name) if File.exist?(name) }
sender = Socket.new(:UNIX, :DGRAM)
sender.bind(Socket.sockaddr_un(first))
listener = Socket.new(:UNIX, :DGRAM)
listener.bind(Socket.sockaddr_un(second))
sender.send('a datagram', 0, Socket.sockaddr_un(second))
message, from = UNIXSocket.for_fd(listener.fileno).recvfrom(10)
p message
p from[0]
p from[1] == first
sender.close
listener.close
[first, second].each { |name| File.delete(name) if File.exist?(name) }
