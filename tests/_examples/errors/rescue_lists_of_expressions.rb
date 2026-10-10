# A rescue list holds expressions as well as names: a ternary, a group, a
# local holding a class, and a name written on a line of its own after a
# comment. Each is read where the clause is reached.
def report
  yield
rescue ArgumentError,
       # a class chosen while the program runs
       defined?(Missing) ? Missing : IOError,
       KeyError => error
  puts("caught #{error.class}")
end

report { raise(IOError) }
report { raise(KeyError) }

chosen = TypeError
begin
  raise(TypeError, "held in a local")
rescue (true ? chosen : IOError) => error
  puts(error.message)
end

begin
  raise(ZeroDivisionError)
rescue IOError, chosen
  puts("not this one")
rescue
  puts("bare rescue")
end

begin; raise(IOError); rescue; puts("rescued on one line"); end
