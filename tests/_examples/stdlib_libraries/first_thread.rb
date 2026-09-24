# On Linux the program runs on the process's first thread, with no other
# thread beside it, so a call such as `setgroups` has no thread to signal.
p(Dir.children("/proc/self/task").length)
