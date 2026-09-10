require 'net/http'
require 'stringio'

# A request writes itself onto a buffered socket as a request line, its
# headers, and the body it carries.
wire = StringIO.new(+"")
socket = Net::BufferedIO.new wire

request = Net::HTTP::Post.new("/orders", "Content-Type" => "text/plain")
request.body = "maintenance window moves to 02:00"
request.exec socket, "1.1", "/orders"
p wire.string.split "\r\n"

streamed = StringIO.new(+"")
request = Net::HTTP::Put.new("/orders", "Content-Type" => "text/plain",
                                        "Transfer-Encoding" => "chunked")
request.body_stream = StringIO.new "chunk me"
request.exec(Net::BufferedIO.new(streamed), "1.0", "/orders")
p streamed.string.split "\r\n"

# A stream with neither a length nor chunked framing has no way to be sent.
begin
  bare = Net::HTTP::Put.new("/orders", "Content-Type" => "text/plain")
  bare.body_stream = StringIO.new "chunk me"
  bare.exec(Net::BufferedIO.new(StringIO.new(+"")), "1.1", "/orders")
rescue ArgumentError => problem
  p problem.class
end

p request.request_body_permitted?
p request.response_body_permitted?
p Net::HTTP::Trace.new("/orders").inspect
