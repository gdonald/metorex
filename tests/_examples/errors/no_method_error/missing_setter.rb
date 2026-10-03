# Assigning through a setter an object does not define raises NoMethodError,
# the same error a missing method of any other name raises.
class Reading
  attr_reader :value

  def initialize
    @value = 1
  end
end

reading = Reading.new
begin
  reading.value = 2
rescue NoMethodError => error
  puts error.message
  puts error.name.inspect
  puts error.receiver.equal?(reading)
end
puts reading.value
