# A thread that never waits on anything still hands the turn over after a
# while, so the main thread runs again and can stop it.

count = 0
spinner = Thread.new { loop { count += 1 } }
Thread.pass until count > 0
p(spinner.alive?)
spinner.kill
spinner.join
p(spinner.status)

stepped = 0
stepper = Thread.new do
  index = 0
  while index < 30_000
    index += 1
    stepped = index
  end
  :finished
end
Thread.pass until stepped > 0
p(stepper.value)
