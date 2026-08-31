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

puts greeter.instance_variable_set "@greeting", "hi"
puts greeter.instance_variable_set :@greeting, "hey"
puts greeter.instance_variable_set GreetingName.new, "howdy"
puts greeter.instance_variable_get :@greeting

emoji = Object.new
emoji.instance_variable_set :@💙, 42
puts emoji.instance_variable_get :@💙

["@", "@0", "@@greeting", "greeting"].each do |name|
  begin
    greeter.instance_variable_set(name, 1)
  rescue NameError => error
    puts "#{error.class}: #{error.message}"
  end
end

begin
  "".instance_variable_set(:greeting, 1)
rescue NameError => error
  puts "#{error.class}: #{error.message}"
end

begin
  nil.instance_variable_set(:@greeting, 1)
rescue FrozenError => error
  puts "#{error.class}: #{error.message}"
end

begin
  greeter.instance_variable_set(10, 1)
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end
