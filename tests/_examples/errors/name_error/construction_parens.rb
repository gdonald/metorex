named = NameError.new("msg", "name")
puts(named.message)
p named.name
begin
  named.receiver
rescue ArgumentError => error
  puts(error.message)
end

symbol_named = NameError.new("msg", :name, receiver: "the receiver")
p symbol_named.name
p symbol_named.receiver

plain = NameError.new("just a message")
puts(plain.message)
p plain.name

class Caller
  def trigger
    missing_helper
  rescue NameError => error
    error
  end
end

caught = Caller.new.trigger
p caught.name
p caught.receiver.class.to_s

copied = caught.dup
p copied.name
p copied.receiver.class.to_s
