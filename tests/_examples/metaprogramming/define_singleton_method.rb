mod = Module.new do
  define_singleton_method :a do
    42
  end
  define_singleton_method(:b, -> x { 2 * x })
end

puts mod.a
puts mod.b(10)

klass = Class.new do
  define_singleton_method(:build) { "built" }
end

puts klass.build

obj = Object.new
obj.define_singleton_method(:greet) { "hi" }
puts obj.greet
puts Object.new.respond_to?(:greet)
