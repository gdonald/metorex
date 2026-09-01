class Described < StandardError
  def to_s
    "this is from to_s"
  end
end

class Quiet < StandardError
end

puts(Exception.new.message)
puts(Exception.new("Ouch!").message)
puts(Described.new("you will not see this").message)
puts(Quiet.new.message)
puts(Quiet.new("plain").message)

error = Exception.new
def error.to_s
  "from a singleton"
end
puts(error.message)

raised = begin
  raise Described
rescue => caught
  caught
end
puts(raised.message)
