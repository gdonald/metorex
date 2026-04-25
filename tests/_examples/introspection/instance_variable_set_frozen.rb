def try_set(obj, label)
  begin
    obj.instance_variable_set(:@spec, "value")
    puts "#{label}: no error"
  rescue FrozenError => e
    puts "#{label}: #{e.class}"
  end
end

try_set(true, "true")
try_set(false, "false")
try_set(nil, "nil")
try_set(42, "integer")
try_set(:sym, "symbol")

class Foo; end
f = Foo.new
f.instance_variable_set(:@x, 99)
puts "instance: #{f.instance_variable_get(:@x)}"

f.freeze
begin
  f.instance_variable_set(:@x, 100)
  puts "frozen instance: no error"
rescue FrozenError
  puts "frozen instance: FrozenError"
end

begin
  true.instance_variable_set(:@bad, 1)
rescue RuntimeError
  puts "rescued via RuntimeError"
end
