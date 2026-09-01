built = NoMethodError.new("msg", "name", ["args"])
p built.args
p built.name

bare = NoMethodError.new("msg")
puts(bare.message)
p bare.args

class Receiver
end

def caught
  yield
rescue NoMethodError => error
  error
end

no_arguments = caught { Receiver.new.missing }
p no_arguments.name
p no_arguments.args

with_arguments = caught { Receiver.new.missing(1, :two, "three") }
p with_arguments.args

copied = with_arguments.dup
p copied.name
p copied.args
p copied.receiver.class.to_s

puts("ab" * 3)
puts("-" * 5)

begin
  puts("x" * -1)
rescue ArgumentError => error
  puts(error.message)
end
