# A C extension making and raising exceptions, reading and setting the
# exception rb_errinfo holds, and naming core classes through rb_e* globals.
require "tmpdir"
require_relative "build_helper"

class Wording
  def to_str
    "worded"
  end
end

class Looping
  def initialize(extension)
    @extension = extension
  end

  def inspect
    @extension.frozen(self)
  end
end

class Pretender
  def exception *
    "not an exception"
  end
end

directory = Dir.mktmpdir
require build_extension("c_exceptions.c", "c_exceptions", directory)
exceptions = CExceptions.new

begin
  raise "outer"
rescue
  p exceptions.info
  p $!.message
end
problem = KeyError.new "held"
p exceptions.set_info(problem).equal? problem
p exceptions.set_info nil
report { exceptions.set_info "text" }

p exceptions.from_bytes "bytes"
p exceptions.from_c_string "c string"
p exceptions.from_string Wording.new
report { exceptions.from_string 5 }
report { exceptions.raise_it IndexError.new("raised from C") }
begin
  begin
    raise "first"
  rescue
    exceptions.raise_it RangeError.new("second")
  end
rescue => raised
  p [raised.message, raised.cause.message]
end

report { exceptions.frozen [1] }
report { exceptions.frozen Looping.new(exceptions) }
begin
  exceptions.frozen "text"
rescue FrozenError => refused
  p refused.receiver
end

p exceptions.system_error(Errno::ENOENT::Errno, nil)
p exceptions.system_error(Errno::ENOENT::Errno, "custom")
p exceptions.system_error_string(Errno::EACCES::Errno, nil).class
p exceptions.system_error_string(Errno::EACCES::Errno, "custom").message

p exceptions.make []
p exceptions.make ["plain"]
p exceptions.make [Wording.new]
held = StandardError.new "kept"
p exceptions.make([held]).equal? held
p exceptions.make [ArgumentError, "with class"]
p exceptions.make([ArgumentError, "with trace", ["here:1"]]).backtrace
report { exceptions.make [nil] }
report { exceptions.make [Object.new] }
report { exceptions.make [Pretender.new] }
report { exceptions.make [ArgumentError, "a", [], "extra"] }

p exceptions.classes
p exceptions.classes.last < Exception

FileUtils.rm_rf directory
