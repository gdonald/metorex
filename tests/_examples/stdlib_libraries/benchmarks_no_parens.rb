# Benchmark measures blocks and prints reports of the times they took. The
# times change from run to run, so the reports are printed with each time
# written as N.
require "benchmark"
require "stringio"

def masked
  captured = StringIO.new
  $stdout = captured
  answer = yield
  $stdout = STDOUT
  print captured.string.gsub(/\d+\.\d{6}/, "N")
  answer
end

fixed = Benchmark::Tms.new(1.5, 0.25, 0.125, 0.0625, 2.0, "fixed")
p fixed.total, fixed.label, fixed.to_a, fixed.to_h
print fixed.to_s.gsub(/\d+\.\d{6}/, "N")
print fixed.format("%n: %5.2u|%5.2y|%5.2U|%5.2Y|%5.2t|%5.2r %s\n", "extra")
sum = fixed + Benchmark::Tms.new(1, 1, 1, 1, 1)
p sum.to_a, (fixed * 2).real, (fixed / 2).utime, (fixed - fixed).total, sum.label
p Benchmark::CAPTION, Benchmark::FORMAT, Benchmark::VERSION
measured = Benchmark.measure("work") { 100.times { } }
p measured.class, measured.label, measured.real.class, measured.real >= 0
p Benchmark.realtime { }.class, Benchmark.ms { }.class
p Benchmark.measure { }.label
results = masked do
  Benchmark.bm(7, ">total:") do |x|
  first = x.report("first:") { 10.times { } }
  second = x.report("second:") { 20.times { } }
  [first + second]
  end
end
p results.size, results.map(&:label)
reports = masked do
  Benchmark.bmbm do |x|
  x.report("one") { 5.times { } }
  x.report("longer label") { 5.times { } }
  end
end
p reports.map(&:label)
added = Benchmark::Tms.new.add { }
p added.class
grown = Benchmark::Tms.new(1, 0, 0, 0, 1)
p grown.add! { }.equal?(grown), grown.utime >= 1
p((Benchmark::Job.new(0).item("x") rescue $!.message))
p Benchmark.private_instance_methods(false).sort
