# A lock another fiber of the same thread holds cannot be waited for.
lock = Mutex.new
lock.lock
begin
  Fiber.new { lock.lock }.resume
rescue ThreadError => error
  p error.message
end
lock.unlock

holder = Fiber.new { lock.lock; Fiber.yield; lock.unlock }
holder.resume
begin
  Fiber.new { lock.lock }.resume
rescue ThreadError => error
  p error.message
end
holder.resume
p lock.locked?
