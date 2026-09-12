# `$!` names the exception being handled. A rescue clause inside a handler
# leaves the outer exception in place once it is done, so the handler that
# follows still names what it is handling.
begin
  raise("outer")
rescue RuntimeError
  p($!.message)
  begin
    raise(ArgumentError, "inner")
  rescue ArgumentError
    p($!.message)
  end
  p($!.message)
  p($!.class)
end
p($!)

# A rescue in a method called from a handler leaves the same thing standing.
def swallowing
  Integer("not a number")
rescue ArgumentError
  :swallowed
end

begin
  raise("again")
rescue RuntimeError
  p(swallowing)
  p($!.message)
end
