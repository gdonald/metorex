module Named
end

puts(Named.to_s)
puts(String.to_s)
puts(Module.new.to_s.start_with?("#<Module:0x").inspect)
puts(Class.new.to_s.start_with?("#<Class:0x").inspect)
puts(Named.singleton_class.to_s)
puts(String.singleton_class.to_s)

anonymous = Class.new
object = anonymous.new
puts((object.singleton_class.to_s == "#<Class:#{object}>").inspect)
puts(object.to_s.start_with?("#<#<Class:0x").inspect)

module Refiner
  Upcase = refine String do
  end
end

puts(Refiner::Upcase.name.inspect)
puts(Refiner::Upcase.to_s)
