# An ensure clause that leaves by `return` drops the exception in flight, so
# `$!` names whatever it named before the begin.

def dropped
  begin
    raise("lost")
  ensure
    return(:returned)
  end
end

p(dropped)
p($!)

begin
  raise("outer")
rescue
  p(dropped)
  p($!.message)
end
