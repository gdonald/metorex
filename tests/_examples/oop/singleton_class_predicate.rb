class Holder
end

instance = Holder.new

puts Holder.singleton_class?.inspect
puts Holder.singleton_class.singleton_class?.inspect
puts instance.singleton_class.singleton_class?.inspect
puts NilClass.singleton_class?.inspect
puts TrueClass.singleton_class?.inspect
puts FalseClass.singleton_class?.inspect
puts Module.new.singleton_class?.inspect

opened = class << instance
  self
end
puts opened.singleton_class?.inspect
