# Taking from an empty queue waits for something to be put there, and a limit
# on how long to wait answers nothing once it passes.
line = Queue.new
p(line.pop(timeout: 0))

line.push(:first)
p(line.pop(timeout: 0))

# Asking for what is there right now rather than waiting is an error when
# there is nothing there.
begin
  line.pop(true)
rescue ThreadError => trouble
  p(trouble.message)
end

# A queue made with a limit takes nothing more while it is full, so a thread
# putting on it waits for a reader and is counted while it waits.
held = SizedQueue.new(1)
held.push(:one)
filling = Thread.new do
  held.push(:two)
end

Thread.pass until filling.stop?
p(held.num_waiting)
p(held.pop)
filling.join
p(held.pop)
