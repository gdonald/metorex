held = Mutex.new
holder = Thread.new { held.lock; Thread.stop }
Thread.pass until holder.status == "sleep"
begin
  held.lock
rescue Exception => error
  p(error.class)
  puts(error.message.lines.first)
end
