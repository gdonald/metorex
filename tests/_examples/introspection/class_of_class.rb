module Greeting
end

class Greeter
end

puts String.class
puts BasicObject.class
puts Greeter.class
puts Class.class
puts Module.class
puts Greeting.class
puts Greeter.new.class

puts String.class.equal? Class
puts Greeting.class.equal?(Module)
