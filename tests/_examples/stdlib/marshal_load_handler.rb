# encoding: binary
seen = []
loaded = Marshal.load(Marshal.dump("foo"), proc { |held| seen << held; held.upcase }, freeze: true)
puts seen.inspect
puts loaded
puts loaded.frozen?

generator = Random.new(42)
generator.rand(100)
copy = Marshal.load(Marshal.dump(generator))
puts copy == generator
puts copy.rand(1000) == generator.rand(1000)
puts copy.seed
puts Marshal.load(Marshal.dump(Random.new(42))) == Random.new(43)

precise = Time.at(0, 123456789, :nsec).utc
dumped = Marshal.dump(precise)
puts dumped.inspect
loaded = Marshal.load(dumped)
puts loaded.nsec
puts loaded.utc?
puts Marshal.load(Marshal.dump(Time.at(0, Rational(1, 3), :nsec))).subsec.inspect
