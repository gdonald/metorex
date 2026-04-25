m = Module.new do
  def self.append_features(mod)
    $appended_to = mod
  end
end

c = Class.new do
  include m
end

puts $appended_to.equal?(c)

frozen = Module.new.freeze
begin
  Module.new.send :append_features, frozen
rescue FrozenError
  puts "frozen ok"
end

a = Module.new
b = Module.new
b.send :append_features, a
begin
  a.send :append_features, b
rescue ArgumentError
  puts "cyclic ok"
end

stub = Module.instance_method(:append_features)
begin
  stub.bind(Class.new).call(Module.new)
rescue TypeError
  puts "rebind ok"
end
