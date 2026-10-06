held = Mutex.new
held.lock
waiter = Thread.new { held.lock; :locked }
begin
  p(waiter.value)
rescue Exception => error
  p(error.class)
  puts(error.message.lines.first)
end
held.unlock
p(waiter.value)

begin
  Thread.stop
rescue ThreadError => error
  p(error.message)
end

Thread.new { sleep(0.05); Thread.main.wakeup }
sleep
puts("main woken")

busy = Mutex.new
busy.lock
blocked = Thread.new { busy.synchronize { :never } }
Thread.pass until blocked.status == "sleep"
blocked.kill
p(blocked.join.status)
p(busy.locked?)
