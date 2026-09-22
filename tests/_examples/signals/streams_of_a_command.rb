# A command run with its error stream pointed at its output writes both into
# the one stream, in the order they were written, and a command ended by a
# signal says which signal ended it.

written = IO.popen(["/bin/sh", "-c", "echo to_the_output; echo to_the_error 1>&2"],
                   err: [:child, :out], &:read)
p(written)
p($?.exitstatus)

# Without the redirect, only what the command wrote to its output is read.
p(IO.popen(["/bin/sh", "-c", "echo only_the_output"], &:read))

ended = IO.popen(["/bin/sh", "-c", "kill -TERM $$"], err: [:child, :out], &:read)
p(ended)
p($?.termsig == Signal.list["TERM"])
p($?.exitstatus)
