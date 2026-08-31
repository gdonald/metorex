class Greeter
  def initialize
    @greeting = "hello"
  end
end

class GreetingName
  def to_str
    "@greeting"
  end
end

greeter = Greeter.new

puts greeter.instance_variable_get "@greeting"
puts greeter.instance_variable_get :@greeting
puts greeter.instance_variable_get GreetingName.new
puts greeter.instance_variable_get(:@goodbye).inspect
puts nil.instance_variable_get(:@goodbye).inspect

["@", "@0", "@@greeting", "greeting"].each do |name|
  begin
    greeter.instance_variable_get(name)
  rescue NameError => error
    puts "#{error.class}: #{error.message}"
  end
end

begin
  greeter.instance_variable_get(10)
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end
