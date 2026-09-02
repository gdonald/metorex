io = IO.popen("echo hello")
print(io.read)
puts(io.read.empty?)
puts(io.pid > 0)
puts(io.closed?)
io.close
puts(io.closed?)

status = Process.last_status
puts(status.exited?)
puts(status.exitstatus)
puts(status.signaled?)
puts(status.termsig.inspect)
puts(status.success?)
puts(status.to_i)

puts(IO.popen("printf one") { |child| child.read })

merged = IO.popen("printf err 1>&2", err: [:child, :out]) { |child| child.read }
puts(merged)

IO.popen("exit 3") { |child| child.read }
puts(Process.last_status.exitstatus)
puts(Process.last_status.success?)
