start = Dir.pwd

Dir.chdir "/tmp" do
  puts Dir.pwd == "/tmp" || Dir.pwd == "/private/tmp"
end

puts Dir.pwd == start

Dir.chdir "/tmp"
puts Dir.pwd == "/tmp" || Dir.pwd == "/private/tmp"
Dir.chdir start
puts Dir.pwd == start

begin
  Dir.chdir "/no/such/directory/here"
rescue RuntimeError => error
  puts error.message.start_with? "No such file or directory"
end

answer = Dir.chdir("/tmp") { 42 }
puts answer
