# A backtrace lists the native methods running between the raise and the
# rescue, inside blocks run against another receiver, inside threads and
# inside a Ractor.
def entries_of(raised)
  raised.backtrace.map { |entry| entry[entry.index(File.basename __FILE__)..] }
end

class Plain
  def converts = Integer "x"
end

def calls_missing = nil.no_such_method

begin
  Plain.new.converts
rescue ArgumentError => raised
  p entries_of raised
end

begin
  Object.new.instance_exec { Integer "x" }
rescue ArgumentError => raised
  p entries_of raised
end

begin
  Plain.new.instance_eval { Integer "x" }
rescue ArgumentError => raised
  p entries_of raised
end

begin
  Plain.class_eval { Integer "x" }
rescue ArgumentError => raised
  p entries_of raised
end

[1].each do
  Plain.class_exec 2 do |number| Integer "x" end
rescue ArgumentError => raised
  p entries_of raised
end

[1].each do
  [2].each { calls_missing }
rescue NoMethodError => raised
  p entries_of raised
end

begin
  [1].map { |number| number / 0 }
rescue ZeroDivisionError => raised
  p entries_of raised
end

thread = Thread.new { Object.new.instance_exec { Integer "x" } }
thread.report_on_exception = false
begin
  thread.join
rescue ArgumentError => raised
  p entries_of raised
end

def returns_from_class_eval
  Plain.class_eval { return 5 }
  6
end
p returns_from_class_eval

Warning[:experimental] = false
ractor = Ractor.new { Integer "x" }
begin
  ractor.value
rescue Ractor::RemoteError => raised
  p entries_of raised.cause
end
