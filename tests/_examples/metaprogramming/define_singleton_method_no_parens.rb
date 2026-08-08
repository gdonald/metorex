mod = Module.new do
  define_singleton_method :a do
    42
  end
end

puts mod.a

obj = Object.new
obj.define_singleton_method :shout do
  "hey"
end

puts obj.shout
