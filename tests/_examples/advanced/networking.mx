response = HTTP.get("https://api.example.com/data")
puts response.body

server = HTTPServer.new(port: 3000)
server.route "/hello" do |request|
  Response.new(body: "Hello, World!")
end
server.start
