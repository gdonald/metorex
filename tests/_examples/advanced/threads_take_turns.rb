# A thread hands the turn over once it has run for a time slice, however
# few statements that took, so a counting thread advances while another
# works through slow ones.
count = 0
counter = Thread.new { loop { count += 1 } }
Thread.pass while count.zero?

seen = count
rounds = 0
until count > seen
  7**300_000
  rounds += 1
end
counter.kill

p(rounds.positive?)
p(rounds < 1000)
