# Marshal writes exceptions, ranges, times and wide numbers the way Ruby
# does, and reads them back whole.
failure = RuntimeError.new("disk full")
failure.set_backtrace(["store.rb:12"])
failure.instance_variable_set(:@retries, 3)
p(Marshal.dump(failure))

begin
  begin
    raise ArgumentError, "bad size"
  rescue ArgumentError
    raise RuntimeError, "write failed"
  end
rescue RuntimeError => raised
  back = Marshal.load(Marshal.dump(raised))
  p([back.class, back.message, back.cause.class, back.cause.message])
end

p(Marshal.dump(1..2))
p(Marshal.dump(1...2))
p(Marshal.load(Marshal.dump(3...9)))

moment = Time.utc(2012, 1, 1)
p(Marshal.dump(moment))
p(Marshal.load(Marshal.dump(moment)) == moment)

wide = 2**64
p(Marshal.dump([wide, wide]))

p(Exception.allocate.message)
