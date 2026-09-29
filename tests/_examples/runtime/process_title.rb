# Process.setproctitle changes the command a process listing shows, and
# leaves $0 as it was.
title = "metorex-title-example"
p(Process.setproctitle(title))
p($0 == title)
