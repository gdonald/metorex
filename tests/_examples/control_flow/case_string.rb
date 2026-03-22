# Case statement with string matching
command = "start"

case command
  when "start"
    puts "Starting server"
  when "stop"
    puts "Stopping server"
  when "restart"
    puts "Restarting server"
  else
    puts "Unknown command"
end
