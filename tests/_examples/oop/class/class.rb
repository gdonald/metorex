class Greeter
  def initialize(name)
    @name = name
  end

  def greeting
    "Hello, " + @name + "!"
  end
end

greeter = Greeter.new("Ada")
message = greeter.greeting
puts message
