# Each call below waits on a child process, a file or a socket that is ready
# only later, and a counter thread beside it shows the other threads ran
# while it waited.
require "socket"
require "tmpdir"

def counted name
  $count = 0
  counter = Thread.new { loop { $count += 1; Thread.pass } }
  Thread.pass
  $count = 0
  result = yield
  counter.kill
  counter.join
  puts "#{name}: #{$count > 0} #{result.inspect}"
end

counted("Kernel#sleep") { sleep(0.2).class }
counted("Kernel#system") { system("sleep 0.2") }
counted("Kernel#`") { `sleep 0.2; echo hi` }
counted("IO.popen") { IO.popen("sleep 0.2; echo hi") { |child| child.read } }
counted("IO.popen writing") { IO.popen("sleep 0.2; cat > /dev/null", "w") { |child| child.write("x" * 200_000) } }
counted("Process.wait") { Process.wait(spawn("sleep 0.2")).class }
counted("Process.wait2") { Process.wait2(spawn("sleep 0.2"))[1].success? }
counted("Process.waitpid") { Process.waitpid(spawn("sleep 0.2")).class }
counted("Process.waitpid2") { Process.waitpid2(spawn("sleep 0.2"))[1].success? }
counted("Process.waitall") { spawn("sleep 0.2"); Process.waitall.size }
counted("Process::Status.wait") { Process::Status.wait(spawn("sleep 0.2")).success? }

Dir.mktmpdir do |directory|
  locked = File.join(directory, "locked")
  File.write locked, ""
  holder = File.open(locked)
  holder.flock File::LOCK_EX
  Thread.new do
    sleep 0.2
    holder.flock File::LOCK_UN
  end
  counted("File#flock") { File.open(locked) { |waiting| waiting.flock(File::LOCK_EX) } }

  named = File.join(directory, "named")
  File.mkfifo named
  Thread.new do
    sleep 0.2
    File.write named, "hi\n"
  end
  counted("File.open on a FIFO") { File.open(named) { |fifo| fifo.read } }
  Thread.new do
    sleep 0.2
    File.write named, "hi\n"
  end
  counted("File.read on a FIFO") { File.read(named) }
  Thread.new do
    sleep 0.2
    File.read named
  end
  counted("File.write on a FIFO") { File.write(named, "hi\n") }

  # A FIFO opened while the other threads end partway, and one opened with
  # no other thread at all, are both read once the child writes.
  brief = Thread.new { 3.times { Thread.pass } }
  writer = spawn "sleep 0.2; echo partway > #{named}"
  puts File.open(named) { |fifo| fifo.read }
  Process.wait writer
  brief.join
  writer = spawn "sleep 0.2; echo alone > #{named}"
  puts File.open(named) { |fifo| fifo.read }
  Process.wait writer

  listening = UNIXServer.new(File.join(directory, "socket"))
  Thread.new do
    sleep 0.2
    UNIXSocket.new(listening.path).close
  end
  counted("UNIXServer#accept") { listening.accept.class }
end

server = TCPServer.new("127.0.0.1", 0)
port = server.addr[1]
Thread.new do
  sleep 0.2
  TCPSocket.new("127.0.0.1", port).close
end
counted("TCPServer#accept") { server.accept.class }

Thread.new do
  sleep 0.2
  connected = TCPSocket.new("127.0.0.1", port)
  sleep 0.2
  connected.write "hi"
  connected.close
end
counted("BasicSocket#recv") { server.accept.recv(10) }

Thread.new do
  sleep 0.2
  connected = TCPSocket.new("127.0.0.1", port)
  sleep 0.2
  connected.write "hi"
  connected.close
end
counted("BasicSocket#recvmsg") { server.accept.recvmsg(10)[0] }

datagrams = UDPSocket.new
datagrams.bind "127.0.0.1", 0
Thread.new do
  sleep 0.2
  UDPSocket.new.send("hi", 0, "127.0.0.1", datagrams.addr[1])
end
counted("UDPSocket#recvfrom") { datagrams.recvfrom(10)[0] }

plain = Socket.new(:INET, :STREAM)
plain.bind Addrinfo.tcp("127.0.0.1", 0)
plain.listen 1
Thread.new do
  sleep 0.2
  TCPSocket.new("127.0.0.1", plain.local_address.ip_port).close
end
counted("Socket#accept") { plain.accept[0].class }

# A host name is looked up while the other threads run.
busy = Thread.new { loop { Thread.pass } }
p Addrinfo.getaddrinfo("localhost", nil, :INET).map(&:ip_address).uniq
named_server = TCPServer.new "localhost", 0
named_port = named_server.addr[1]
p TCPSocket.new("localhost", named_port).class
p Socket.tcp("localhost", named_port).class
busy.kill
