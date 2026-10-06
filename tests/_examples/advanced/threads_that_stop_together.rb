stopped = Thread.new { Thread.stop }
Thread.pass until stopped.status == "sleep"
begin
  Thread.stop
rescue Exception => error
  p(error.class)
  puts(error.message.lines.first)
end
