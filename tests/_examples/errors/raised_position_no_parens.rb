# An exception the interpreter raises carries the stack it was raised on,
# which `$@` reads for the one being handled.
held = (1 / 0 rescue $!)
p held.class
p held.backtrace.class
p((1 / 0 rescue $@).class)

# A namespaced error is placed by its ancestors, so `SystemCallError` catches
# the Errno classes under it.
begin
  File.read "/no/such/path/at/all"
rescue SystemCallError => problem
  p problem.class
end
