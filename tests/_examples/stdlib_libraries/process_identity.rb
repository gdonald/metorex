# What a process reports about itself: the groups and sessions it belongs to,
# the scheduling it runs under, and the ids it runs as.
p(Process.getpgrp == Process.getpgid(0))
p(Process.getsid == Process.getsid(0))
p(Process.getpriority(Process::PRIO_PROCESS, 0).is_a?(Integer))
p(Process.times.is_a?(Process::Tms))
p(Process.times.utime.is_a?(Float))

# The same ids go by more than one name.
p(Process::UID.rid == Process.uid)
p(Process::GID.rid == Process.gid)
p(Process::Sys.getuid == Process.uid)
p(Process::Sys.getegid == Process.egid)

# The constants the operating system names its settings by.
p([Process::WNOHANG, Process::WUNTRACED].all? { |held| held.is_a?(Integer) })
p([Process::PRIO_PROCESS, Process::PRIO_PGRP, Process::PRIO_USER].uniq.length)
p(Process::RLIMIT_NOFILE.is_a?(Integer))

# Renaming the process leaves `$0` alone, and forking is not carried here.
p(Process.setproctitle("metorex-example"))
begin
  Process._fork
rescue NotImplementedError => problem
  p(problem.class)
end

# A signal sent to a process that is not there is refused.
begin
  Process.kill(:TERM, 999_999)
rescue Errno::ESRCH => problem
  p(problem.class)
end

# Setting a group list without the right to do so is refused too.
begin
  Process.groups = [0]
rescue Errno::EPERM => problem
  p(problem.class)
end
