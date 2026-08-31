class Greeter
  def initialize
    @greeting = "hello"
    @name = "world"
  end
end

greeter = Greeter.new
puts greeter.instance_variables.inspect
removed = greeter.remove_instance_variable :@greeting
puts removed.inspect
puts greeter.instance_variables.inspect
puts greeter.instance_variable_defined? :@greeting

removed_name = greeter.remove_instance_variable "@name"
puts removed_name.inspect
puts greeter.instance_variables.inspect

begin
  greeter.remove_instance_variable :@unknown
rescue NameError => error
  puts "#{error.class}: #{error.message}"
end

begin
  greeter.remove_instance_variable :"@0"
rescue NameError => error
  puts "#{error.class}: #{error.message}"
end

begin
  greeter.remove_instance_variable Object.new
rescue TypeError => error
  puts error.class
end

class Name
  def to_str
    "@greeting"
  end
end

reborn = Greeter.new
reborn_greeting = reborn.remove_instance_variable Name.new
puts reborn_greeting.inspect

frozen = Greeter.new
frozen.freeze
begin
  frozen.remove_instance_variable :@greeting
rescue FrozenError => error
  puts error.class
end

begin
  nil.remove_instance_variable :@anything
rescue FrozenError => error
  puts error.class
end

begin
  nil.remove_instance_variable :not_a_variable
rescue NameError => error
  puts error.class
end

puts Kernel.public_instance_methods(false).include? :remove_instance_variable
