error = KeyError.new(receiver: "lookup source", key: :b)
p error.receiver
p error.key
puts(error.message)

described = KeyError.new("key not found: :b", receiver: "lookup source", key: :b)
puts(described.message)
p described.key

plain = KeyError.new("no lookup recorded")
begin
  plain.key
rescue ArgumentError => error
  puts(error.message)
end
begin
  plain.receiver
rescue ArgumentError => error
  puts(error.message)
end

frozen = FrozenError.new("can't modify", receiver: "text")
p frozen.receiver
puts(frozen.message)
