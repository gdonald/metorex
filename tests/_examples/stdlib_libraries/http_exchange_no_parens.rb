# Net::HTTP against a server in another thread: a gzipped body arrives
# inflated, a body is read to its Content-Length on a connection kept open,
# and a connection the server closed is opened again for the next request.
require "net/http"
require "zlib"
require "stringio"

server = TCPServer.new "127.0.0.1", 0
serving = Thread.new do
  loop do
    client = server.accept
    while (request = client.gets("\r\n\r\n"))
      verb, path = request.split(" ", 3)
      body = "#{verb} #{path} answered"
      if path == "/zipped"
        packed = StringIO.new
        zipper = Zlib::GzipWriter.new(packed)
        zipper.write body
        zipper.close
        client.write("HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: #{packed.string.bytesize}\r\n\r\n#{packed.string}")
      elsif verb == "HEAD"
        client.write("HTTP/1.1 200 OK\r\n\r\n")
        break
      else
        client.write("HTTP/1.1 200 OK\r\nContent-Length: #{body.bytesize}\r\n\r\n#{body}")
      end
    end
    client.close
  end
end

http = Net::HTTP.start "127.0.0.1", server.addr[1]
p http.get("/zipped").body
p http.get("/plain").body
p http.head("/plain").body
p http.post("/form", "name=value").body
http.finish
serving.kill
