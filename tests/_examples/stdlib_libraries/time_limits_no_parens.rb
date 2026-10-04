# A block given a limit on how long it may run, and the collector's settings.
require "timeout"

p Timeout.timeout(1) { 42 }

begin
  Timeout.timeout 1 do
    sleep
  end
rescue Timeout::Error => expired
  p [expired.class, expired.message]
end

begin
  Timeout.timeout 1, ArgumentError, "took too long" do
    sleep 5
  end
rescue ArgumentError => expired
  p expired.message
end

begin
  Timeout.timeout -1 do
    :never
  end
rescue ArgumentError => refused
  p refused.message
end

# A limit ends with its block, so a later wait is not cut short by it.
p Timeout.timeout(0.01) { :inside }
p sleep 0.05

settings = GC.config
p settings[:implementation].is_a? String
p GC.config(foo: "bar") == settings
