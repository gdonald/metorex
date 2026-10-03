# A C extension calling back into Ruby: methods by name, constants, Marshal,
# finalizers, the integer conversions, and classes and modules defined under
# a module.
require "tmpdir"
require "stringio"
require_relative "build_helper"

class Recorder
  def initialize
    @calls = []
  end

  attr_reader :calls

  def record(first, second)
    @calls << [first, second]
    :recorded
  end

  def binding
    :written_in_ruby
  end
end

class Number
  def initialize(value)
    @value = value
  end

  def to_int
    @value
  end
end

module Outer
  autoload :Loaded, File.join(__dir__, "autoloaded_class.rb")
end

directory = Dir.mktmpdir
require(build_extension("c_calls.c", "c_calls", directory))
calls = CCalls.new

p(calls.symbol_for(:name))
p(calls.same_id(:name, :name))
p(calls.same_id(:name, :other))
report { calls.symbol_for("name") }

recorder = Recorder.new
p(calls.call_two(recorder, :record, 1, "two"))
p(recorder.calls)
p(calls.call_none([3, 1, 2], :sort))
p(calls.call_none(recorder, :binding))
report { calls.call_none(Object.new, :binding) }
report { calls.call_none(1, :no_such_method) }

p(calls.constant(Object, :Comparable))
p(calls.constant(Outer, :Loaded))
report { calls.constant(Object, :NoSuchConstant) }

data = calls.dump("text", nil)
p(data == Marshal.dump("text"))
port = StringIO.new
calls.dump([1, 2], port)
p(port.string == Marshal.dump([1, 2]))
p(calls.load(data))

finalizer = proc { puts "finalizer ran" }
p(calls.define_finalizer(Object.new, finalizer).equal?(finalizer))
report { calls.define_finalizer(Object.new, 1) }
removed = Object.new
ObjectSpace.define_finalizer(removed, proc { puts "removed finalizer ran" })
p(calls.undefine_finalizer(removed).equal?(removed))

p([calls.to_long(42), calls.to_long(-7), calls.to_long(3.99), calls.to_long(Number.new(5))])
report { calls.to_long(nil) }
report { calls.to_long(Float::INFINITY) }
report { calls.to_long(2**64) }
report { calls.to_long("1") }
p([calls.quarter_unsigned_long(-1), calls.quarter_unsigned_long(2**63)])
report { calls.quarter_unsigned_long(2**64) }
p([calls.to_int(2**31 - 1), calls.to_int(-2**31)])
report { calls.to_int(2**31) }
report { calls.to_int(-2**31 - 1) }
p([calls.to_unsigned_int(2**32 - 1), calls.to_unsigned_int(-1)])
report { calls.to_unsigned_int(2**32) }
report { calls.to_unsigned_int(-2**31 - 1) }
p([calls.fix_to_int(-14), calls.fix_to_unsigned_int(42)])

module Holder
end
inner = calls.class_under(Holder, :Inner, Object)
p(inner)
p(calls.class_under(Holder, :Inner, Object).equal?(inner))
report { calls.class_under(Holder, :Inner, String) }
Holder.const_set(:Taken, 1)
report { calls.class_under(Holder, :Taken, Object) }
Parent = Class.new
p(calls.class_id_under(Holder, :ById, Parent))
p(Holder::ById.superclass)
p(calls.class_under(Outer, :Loaded, Object))

top = calls.module(:CTopModule)
p(top)
p(calls.module(:CTopModule).equal?(top))
p(calls.module_under(top, :Nested))
report { calls.module(:String) }

FileUtils.rm_rf(directory)
