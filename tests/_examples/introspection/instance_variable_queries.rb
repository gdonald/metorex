class Greeter
  def initialize
    @greeting = "hello"
  end
end

module Marker
end

greeter = Greeter.new

puts greeter.instance_variable_defined?("@greeting")
puts greeter.instance_variable_defined?(:@greeting)
puts greeter.instance_variable_defined?("@goodbye")
puts greeter.instance_variable_defined?(:@goodbye)

begin
  greeter.instance_variable_defined?(Object.new)
rescue TypeError => error
  puts error.class
end

puts nil.instance_variable_defined?("@goodbye")
puts 1.instance_variable_defined?("@goodbye")
puts "text".instance_variable_defined?("@goodbye")

puts greeter.instance_of?(Greeter)
puts greeter.instance_of?(Object)
puts greeter.instance_of?(Marker)
puts greeter.instance_of?(Comparable)
