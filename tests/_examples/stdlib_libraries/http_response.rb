require 'net/http'
require 'stringio'

# A response reads its status line and headers off a buffered socket, and
# the class it comes back as says what the code means.
raw = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Trace: alpha\r\n\r\nmaintenance window moves to 02:00\n"
socket = Net::BufferedIO.new(StringIO.new(raw))
response = Net::HTTPResponse.read_new(socket)

p response.class
p response.code
p response.message
p response.http_version
p response["content-type"]
p response.to_hash
p response.code_type
p response.error_type
p Net::HTTPOK.body_permitted?
p Net::HTTPInformation.body_permitted?

response.reading_body(socket, true) do
  p response.inspect
end
p response.body
p response.inspect
p response.value

# A response outside the 2xx range raises the exception its class names.
failed = Net::HTTPNotFound.new("1.1", "404", "Not Found")
begin
  failed.value
rescue Net::HTTPClientException => problem
  p problem.class
  p problem.message
  p problem.response.equal?(failed)
end

redirect = Net::HTTPFound.new("1.1", "302", "Found")
begin
  redirect.error!
rescue Net::HTTPRetriableError => problem
  p problem.class
end
