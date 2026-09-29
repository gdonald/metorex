# Process.kill with signal 0 checks that a process can be signaled.
p Process.kill 0, Process.pid
