require "objspace"
require "json"

held = ObjectSpace.dump("abc")
dump = JSON.parse(held)
puts dump["type"]
puts dump["value"]
puts dump["bytesize"]
puts dump["encoding"]

puts JSON.parse(ObjectSpace.dump([1, 2, 3]))["type"]
puts JSON.parse(ObjectSpace.dump({ a: 1 }))["type"]
puts JSON.parse(ObjectSpace.dump(String))["name"]

marker = "a marker"
whole = ObjectSpace.dump_all output: :string
puts whole.include?('"value":"a marker"')

begin
  ObjectSpace.dump "abc", output: Object.new
rescue ArgumentError => refused
  puts refused.message.sub(/0x[0-9a-f]+/, "0x")
end
