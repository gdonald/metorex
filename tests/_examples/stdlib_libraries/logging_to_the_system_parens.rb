# The system log, and a binding as a scope a program can reach into.

require 'syslog'

p(Syslog.opened?)
Syslog.open "metorex_example", Syslog::LOG_PID
p(Syslog.opened?)
p(Syslog.ident)
p(Syslog.options == Syslog::LOG_PID)
p(Syslog.facility == Syslog::LOG_USER)
p(Syslog.mask)
Syslog.mask = Syslog::Constants.LOG_UPTO Syslog::LOG_WARNING
p(Syslog.mask)
Syslog.close
p(Syslog.opened?)
p(Syslog.mask)

# The mask bit one severity stands for.
p(Syslog::Constants.LOG_MASK Syslog::LOG_DEBUG)
p(Syslog::Constants.LOG_MASK Syslog::LOG_WARNING)

# Leaving a log nobody has opened is refused.
begin
  Syslog.close
rescue RuntimeError => problem
  p(problem.message)
end

# A binding names the locals in force where it was taken, and code run
# through it leaves what it binds behind.
first = 1
second = 2
held = binding
p(held.local_variables.include? :first)
p(held.local_variable_get :first)
held.local_variable_set :third, 3
p(held.local_variable_get :third)
p(held.local_variable_defined? :third)
p(held.local_variable_defined? :fourth)
held.eval "fourth = 4"
p(held.local_variable_get :fourth)
p(held.source_location.last > 0)
