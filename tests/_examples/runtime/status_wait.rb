# Process::Status.wait answers a child's status and leaves $? alone.
p(Process::Status.wait().pid)
pid = Process.spawn("true")
status = Process::Status.wait()
p(status.pid == pid)
p(status.class)
p($?.nil?)
