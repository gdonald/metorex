# A child's status reads how it ended: the code it exited with, the signal
# that ended it, or the signal that stopped it.
def described(status)
  [
    status.to_s.sub(/pid \d+/, "pid N").gsub(/\(signal (\d+)\)/) { "(signal #{Signal.signame($1.to_i)})" },
    status.inspect.sub(/pid \d+/, "pid N").gsub(/\(signal (\d+)\)/) { "(signal #{Signal.signame($1.to_i)})" },
    status.exited?, status.exitstatus, status.success?,
    status.signaled?, status.termsig && Signal.signame(status.termsig),
    status.stopped?, status.stopsig && Signal.signame(status.stopsig),
    status.coredump?
  ]
end

system("exit 0")
p(described($?))
system("exit 3")
p(described($?))
p($?.exitstatus)

killed = spawn("sleep 5")
Process.kill(:KILL, killed)
Process.wait(killed)
p(described($?))

stopped = spawn("sleep 5")
Process.kill(:STOP, stopped)
_, status = Process.waitpid2(stopped, Process::WUNTRACED)
p(described(status))
Process.kill(:KILL, stopped)
Process.wait(stopped)

terminated = spawn("sh", "-c", "kill -TERM $$")
Process.wait(terminated)
p(described($?))

p(Process::Status.instance_methods(false).sort)
