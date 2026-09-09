# Constants and readings the core classes carry.
p File::NULL
p File.include?(File::Constants)
p File.superclass
p IO.include?(Enumerable)

p Encoding::BINARY.equal?(Encoding::ASCII_8BIT)
p Encoding::EUC_JP.name
p Encoding.find("ISO-8859-1").name
p Encoding::UTF_8.inspect
p Encoding::ISO_2022_JP.dummy?
p Encoding.list.all? { |found| found == Encoding.find(found.name) }

p Process::Tms.new(1, 2, 3, 4).utime
p Process.warmup
Process.maxgroups = 12
p Process.maxgroups

before = GC.count
GC.start
p GC.count > before
p GC.total_time.is_a?(Integer)
p GC::Profiler.enabled?
p GC::Profiler.result

group = ThreadGroup.new
p group.enclosed?
group.enclose
p group.enclosed?
p Thread.main.group.equal?(ThreadGroup::Default)
p Thread.ignore_deadlock

p Complex(1, 2).send(:marshal_dump)
p Rational(1, 2).send(:marshal_dump)
p Regexp.linear_time?(/a+/)
