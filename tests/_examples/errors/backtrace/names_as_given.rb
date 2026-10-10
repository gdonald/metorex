# A backtrace names the program file the way the command line gave it, and
# a NameError names the line of the bare name it could not find.
begin
  nope
rescue NameError => error
  p(error.backtrace)
end

def go
  missing
end

begin
  go
rescue NameError => error
  p(error.backtrace)
end
