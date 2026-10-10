# Timeout.timeout raises into its block wherever the block stands, so a
# loop that never waits is stopped too. The error is raised once, and the
# block's ensure clauses run as it unwinds.
require "timeout"

begin
  Timeout.timeout(0.05) { loop {} }
rescue Timeout::Error => error
  p([error.class, error.message])
end

begin
  Timeout.timeout(0.05) { true while true }
rescue Timeout::Error => error
  p([error.class, error.message])
end

begin
  Timeout.timeout(0.05, ArgumentError, "slow") do
    count = 0
    count += 1 while true
  end
rescue ArgumentError => error
  p([error.class, error.message])
end

begin
  Timeout.timeout(0.05) do
    loop {}
  ensure
    p(:ensured)
  end
rescue Timeout::Error
  p(:raised_once)
end

begin
  Timeout.timeout(5) { Timeout.timeout(0.05) { loop {} } }
rescue Timeout::Error
  p(:inner_limit)
end

p(Timeout.timeout(1) { 42 })
