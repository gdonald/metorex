# A pattern can be given a limit on how long one match may run. A match that
# passes the limit raises Regexp::TimeoutError rather than running on.
p Regexp.timeout

# A limit written on a pattern of its own stands in place of the one the
# class names, and reading it back answers what was set.
slow = Regexp.new "(a*)*b", timeout: 0.5
p slow.timeout
p slow.source

# A pattern built without one has none of its own, whatever the class says.
plain = Regexp.new "(c*)*d"
p plain.timeout

# The limit has to be a positive number of seconds.
held = begin
  Regexp.new "x", timeout: 0
rescue ArgumentError => error
  error.message
end
p held

held = begin
  Regexp.timeout = -1
rescue ArgumentError => error
  error.message
end
p held

# The class names a limit for every pattern written without one, read back
# as a number of seconds.
Regexp.timeout = 5
p Regexp.timeout
p plain.timeout
p(/(a+)b/.match("aaab")[1])

# Clearing it takes the limit off again.
Regexp.timeout = nil
p Regexp.timeout
p Regexp::TimeoutError.superclass
