# A signal another process sends runs the handler Signal.trap installed.
received = []
Signal.trap(:TERM) { |number| received << number }
system("kill -TERM #{Process.pid}")
sleep(0.01) while received.empty?
p(received)
Signal.trap(:TERM, "DEFAULT")
p(Process.kill(0, Process.pid))
