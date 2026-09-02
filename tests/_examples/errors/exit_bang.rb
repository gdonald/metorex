begin
  puts "before exit!"
  exit! 9
rescue SystemExit
  puts "not rescuable"
ensure
  puts "ensure does not run"
end
