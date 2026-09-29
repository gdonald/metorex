# A thread raising into another that loops on a lock inside a fiber reaches it.
lock = Mutex.new
t2 = nil
t1 = Thread.new do
  loop do
    sleep 0.01
    t2.raise if t2
  end
end

rounds = 0
t2 = Thread.new do
  3.times do
    Fiber.new do
      begin
        loop { lock.synchronize {} }
      rescue RuntimeError
        rounds += 1
      end
    end.resume
  rescue RuntimeError
    retry
  end
end
t2.join
t1.kill
t1.join
p rounds > 0
p lock.locked?
