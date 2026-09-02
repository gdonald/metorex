class Connection
  def initialize
    @host = "localhost"
    @port = 5432
    @open = true
  end
end

connection = Connection.new
puts connection.inspect.sub(/0x[0-9a-f]+/, "0x")
puts connection.to_s.sub(/0x[0-9a-f]+/, "0x")

class Bare
end

puts Bare.new.inspect.sub(/0x[0-9a-f]+/, "0x")

class Chosen
  def initialize
    @shown = "yes"
    @hidden = "no"
  end

  def instance_variables_to_inspect = [:@shown, :@missing]
end

puts Chosen.new.inspect.sub(/0x[0-9a-f]+/, "0x")

class NoneChosen
  def initialize
    @a = 1
  end

  def instance_variables_to_inspect = []
end

puts NoneChosen.new.inspect.sub(/0x[0-9a-f]+/, "0x")

class BadChoice
  def initialize
    @a = 1
  end

  def instance_variables_to_inspect = {}
end

begin
  BadChoice.new.inspect
rescue TypeError => error
  puts error.message
end
