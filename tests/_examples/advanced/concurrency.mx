channel = Channel.new

def expensive_computation()
  sleep 1
end

thread = Thread.new do
  result = expensive_computation()
  channel.send(result)
end

result = channel.receive
puts "Got result: #{result}"
