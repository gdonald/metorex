require 'socket'
require 'net/ftp'

# A server answering the handful of commands a listing and a download need.
listener = TCPServer.new('127.0.0.1', 0)
serving = Thread.new do
  control = listener.accept
  control.print("220 ready\r\n")
  data = nil
  while (line = control.gets)
    line = line.chomp
    case line
    when /\APORT (.*)\z/
      parts = Regexp.last_match(1).split(',')
      data = TCPSocket.new(parts[0, 4].join('.'), parts[4].to_i * 256 + parts[5].to_i)
      control.print("200 port opened\r\n")
    when /\ALIST/
      control.print("150 opening\r\n")
      data.print("one.rb\r\n")
      data.print("two.rb\r\n")
      data.close
      control.print("226 transfer complete\r\n")
    when /\ARETR/
      control.print("125 sending\r\n")
      data.print("first line\r\nsecond line\r\n")
      data.close
      control.print("226 transfer complete\r\n")
    when /\AMDTM/
      control.print("213 19980705132316\r\n")
    when /\AQUIT/
      control.print("221 bye\r\n")
      break
    else
      control.print("200 ok\r\n")
    end
  end
  control.close
end

session = Net::FTP.new
session.passive = false
session.connect('127.0.0.1', listener.addr[1])

p(session.list('files'))

lines = []
session.retrlines('RETR notes.txt') { |line| lines << line }
p(lines)

p(session.mtime('notes.txt'))
p(session.quit)

serving.join
listener.close
