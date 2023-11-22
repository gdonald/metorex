def custom_wrapper(name, block)
  puts "Starting: #{name}"

  block.call

  puts "Finished: #{name}"
end

custom_wrapper "Network Fetch" do
  Network.fetch "data.json"
end
