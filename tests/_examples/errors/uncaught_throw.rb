# `UncaughtThrowError#tag` and `#value` report the throw that found no catch.
begin
  throw :abc
rescue UncaughtThrowError => error
  p error.tag
  p error.value
  puts error.message
end

begin
  catch :a do
    throw :b, "carried"
  end
rescue UncaughtThrowError => error
  p error.tag
  p error.value
end

# Both are nil on an UncaughtThrowError no throw raised.
p UncaughtThrowError.new("uncaught throw :abc").tag
p UncaughtThrowError.new("uncaught throw :abc").value

# Another exception answers neither.
p RuntimeError.new("x").respond_to? :tag
