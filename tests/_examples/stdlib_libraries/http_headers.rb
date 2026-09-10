require 'net/http'

# A request carries its headers under case-insensitive names, and reads
# several values under one name back as a joined string.
request = Net::HTTP::Post.new "/orders"
request["Content-Type"] = "text/html; charset=utf-8"
request.add_field "X-Trace", "alpha"
request.add_field "X-Trace", "beta"

p request.method
p request.path
p request["content-type"]
p request["X-TRACE"]
p request.get_fields("x-trace")
p request.main_type
p request.sub_type
p request.content_type
p request.type_params
p request.key?("X-Trace")
p request.fetch("missing", "fallback")

request.content_length = 42
p request.content_length
request.range = 10..200
p request["Range"]
p request.range
request["Content-Range"] = "bytes 0-499/1234"
p request.content_range
p request.range_length

request.basic_auth "maintenance", "window"
p request["Authorization"]

request.set_form_data "cmd" => "search", "max" => "50"
p request.body.split("&").sort
p request["Content-Type"]

names = []
request.each_capitalized_name { |name| names << name }
p names.sort.include?("X-Trace")

p request.delete("X-Trace")
p request.key?("x-trace")
p request.chunked?
request["Transfer-Encoding"] = "chunked"
p request.chunked?
