# Writes its pid to the file named, then waits for a TERM and says it got one.
Signal.trap(:TERM) { $signaled = true }
File.write(ARGV.shift, Process.pid.to_s)
sleep(0.001) until $signaled
puts("signaled")
