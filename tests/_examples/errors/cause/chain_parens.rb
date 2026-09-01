# `Exception#cause` is the exception a rescue clause was handling when this
# one was raised, nil when there was none, and never the exception itself.

standalone = RuntimeError.new("standalone")
puts(standalone.cause.inspect)

begin
  raise Exception, "the cause"
rescue Exception
  begin
    raise RuntimeError, "the consequence"
  rescue RuntimeError => error
    puts(error.message)
    puts(error.cause.class.to_s)
    puts(error.cause.message)
  end
end

# An error raised inside a rescue body records what it followed, including one
# the interpreter itself raises.
begin
  begin
    1 / 0
  rescue ZeroDivisionError
    raise("followed a division")
  end
rescue RuntimeError => error
  puts(error.cause.class.to_s)
  puts(error.cause.is_a?(ZeroDivisionError).to_s)
  puts(error.cause.is_a?(StandardError).to_s)
end

original = RuntimeError.new("original")
begin
  begin
    raise(original)
  rescue RuntimeError
    1 / 0
  end
rescue ZeroDivisionError => error
  puts(error.cause.equal?(original).to_s)
end

# Re-raising the same exception leaves its cause alone.
begin
  begin
    raise RuntimeError, "re-raised"
  rescue RuntimeError => error
    raise(error)
  end
rescue RuntimeError => error
  puts(error.cause.inspect)
end
