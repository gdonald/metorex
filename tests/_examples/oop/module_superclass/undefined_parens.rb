# A module has no superclass and does not answer `superclass`, while every
# class does.
module Greetings
end

puts(Greetings.respond_to?(:superclass))
begin
  Greetings.superclass
rescue NoMethodError => error
  puts(error.message)
end
puts(Class.new.superclass.inspect)
puts(BasicObject.superclass.inspect)
