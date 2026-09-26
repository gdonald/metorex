# A standard stream's constant may be pointed at another stream inside a
# block and pointed back afterwards, which leaves the original open.
original = STDERR
[1].each() do
  $VERBOSE = nil
  STDERR = File.open("/dev/null", "w")
  STDERR.close()
  begin
    p(STDERR.closed?())
  ensure
    STDERR = original
  end
end
p(STDERR.equal?(original))
p(STDERR.closed?())
p($stderr.closed?())
