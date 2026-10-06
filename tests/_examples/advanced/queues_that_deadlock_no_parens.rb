empty = Queue.new
begin
  empty.pop
rescue Exception => error
  p error.class
  puts error.message.lines.first
end
