# Process.daemon leaves the process that called it, and carries on as a
# process of its own in / with its standard streams on /dev/null.
require "rbconfig"
require "tmpdir"
report = File.join(Dir.tmpdir, "metorex_daemonizing_#{Process.pid}.txt")
code = <<~CODE
  started = Process.pid
  at_exit { File.write(#{report.inspect}, [started != Process.pid, Process.getpgrp == Process.pid, Dir.pwd, STDIN.stat.chardev?].inspect) }
  puts("before")
  Process.daemon
  puts("after")
CODE
output = IO.popen([RbConfig.ruby, "-e", code]) { |pipe| pipe.read }
sleep(0.01) until File.exist?(report) && File.size?(report)
p(output)
p(File.read(report))
File.delete(report)
