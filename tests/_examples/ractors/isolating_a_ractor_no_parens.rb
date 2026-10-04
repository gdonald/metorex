# A Ractor other than the main one may not reach the main program's locals,
# its globals, its class variables, or a constant holding an object that is
# not shareable. `Ractor.make_shareable` freezes an object and everything it
# refers to so any Ractor may hold it.

Warning[:experimental] = false
Thread.report_on_exception = false

def attempt
  yield
rescue Exception => trouble
  [trouble.class, trouble.message.gsub(/0x\h+/, "ADDRESS").sub(/ \S+\.rb:\d+/, " WRITTEN_AT")]
end

first = 1
second = 2
p attempt { Ractor.new { first + second } }
p attempt { Ractor.new { first = 5 } }
p Ractor.new { counted = 1; [1].map { |step| step + counted } }.value
p Ractor.new { [1].map { |first| first } }.value

$setting = 1
class Box
  @@count = 1
  def self.count = @@count
  LIST = [1, 2]
  FROZEN = [1, 2].freeze
end
LIST = [3]

reached = Ractor.new do
  [
    attempt { $setting },
    attempt { $setting = 2 },
    attempt { $stdout.class },
    attempt { $0 },
    attempt { Box.count },
    attempt { Box::LIST },
    attempt { Box::FROZEN },
    attempt { LIST },
    attempt { ::LIST },
    attempt { Box.const_set(:ADDED, []) }
  ]
end
reached.value.each { |line| p line }

p [1, :a, nil, 1.5, String, "text".freeze, 1..2].map { |held| Ractor.shareable?(held) }
p [+"text", [1, +"x"].freeze, Object.new].map { |held| Ractor.shareable?(held) }

nested = [1, [+"deep", { key: +"value" }], +"top"]
p Ractor.make_shareable(nested).equal?(nested)
p [nested.frozen?, nested[1].frozen?, nested[1][0].frozen?, nested[1][1][:key].frozen?]
p Ractor.shareable?(nested)

original = [+"x"]
copied = Ractor.make_shareable(original, copy: true)
p [copied.equal?(original), original.frozen?, copied.frozen?]

p attempt { Ractor.make_shareable(proc { 1 }) }
text = +"open"
p attempt { Ractor.make_shareable(nil.instance_eval { proc { text } }) }
count = 1
p attempt { Ractor.make_shareable(nil.instance_eval { proc { count = 2 } }) }
p attempt { Ractor.make_shareable(nil.instance_eval { proc { count + 1 } }) }
base = 1
summed = Ractor.make_shareable(nil.instance_eval { proc { base + 1 } })
p [summed.call, summed.frozen?, Ractor.shareable?(summed), Ractor.shareable?(proc { 1 })]

shared = Ractor.shareable_proc { 1 + 2 }
p [shared.call, Ractor.shareable?(shared)]
shared_lambda = Ractor.shareable_lambda { 3 }
p [shared_lambda.lambda?, shared_lambda.call]
p attempt { Ractor.make_shareable(Thread::Mutex.new) }
