# A signal a program sends itself arrives inside the call that sent it, and an
# Interrupt carries the number of the signal it stands for.

begin
  Process.kill(:INT, Process.pid)
  sleep
rescue Interrupt => arrived
  p(arrived.signo == Signal.list["INT"])
  p(["", "Interrupt"].include?(arrived.message))
  p(arrived.backtrace.first.include?("Process.kill"))
end

# A command run with its error stream pointed at its output writes both into
# the one stream, in the order they were written.
written = IO.popen(["/bin/sh", "-c", "echo to_the_output; echo to_the_error 1>&2"],
                   err: [:child, :out], &:read)
p(written)
p($?.exitstatus)
