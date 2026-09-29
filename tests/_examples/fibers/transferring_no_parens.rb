# A fiber reached by a transfer from the top level hands what it answered
# back to the top level.
first = Fiber.new { :first }
second = Fiber.new { first.transfer; :second }
p second.transfer
p second.transfer

# One reached by a transfer inside a resumed fiber hands it to that fiber.
finisher = Fiber.new { :finisher }
starter = Fiber.new { Fiber.yield finisher.transfer; :unreached }
p starter.resume

# A fiber waiting in Fiber.yield takes no transfer.
begin
  starter.transfer
rescue FiberError => error
  p error.message
end

# Transferring to the fiber running now leaves it there.
p Fiber.current.transfer 1, 2
root = Fiber.current
p Fiber.new { root.transfer :back }.transfer

# A dead fiber takes no transfer.
done = Fiber.new { 1 }
done.transfer
begin
  done.transfer
rescue FiberError => error
  p error.message
end

# A fiber part-way through resuming another takes no transfer.
outer = nil
inner = Fiber.new { outer.transfer }
outer = Fiber.new { inner.resume }
begin
  outer.resume
rescue FiberError => error
  p error.message
end
begin
  Fiber.new { root.transfer }.resume
rescue FiberError => error
  p error.message
end

# A fiber made on one thread takes no transfer from another.
here = Fiber.new { 42 }
Thread.new do
  begin
    here.transfer
  rescue FiberError => error
    p error.message
  end
end.join
p here.transfer

# A thread's own fiber and one it made hand control back and forth.
Thread.new do
  helper = Fiber.new { |calling| calling.transfer :from_helper; :helper_done }
  p helper.transfer Fiber.current
  p helper.transfer
end.join

# A fiber resumed inside another can transfer to the top level, and a
# transfer back to it later finishes both.
low = Fiber.new { answer = root.transfer :up; [answer, :low_done] }
high = Fiber.new { [low.resume, :high_done] }
p high.transfer
p [low.alive?, high.alive?]
begin
  high.transfer
rescue FiberError => error
  p error.message
end
p low.transfer :down
p [low.alive?, high.alive?]
