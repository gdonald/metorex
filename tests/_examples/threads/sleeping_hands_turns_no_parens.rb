# A thread that sleeps for a length lets the other threads run until the
# length passes, and a sleep is cut short by a wakeup.

log = []
sleeper = Thread.new do
  log << :sleeper_start
  sleep 0.05
  log << :sleeper_end
end
worker = Thread.new { log << :worker }
[sleeper, worker].each(&:join)
p log


napper = Thread.new { sleep 5 }
Thread.pass until napper.status == "sleep"
napper.wakeup
p napper.value
