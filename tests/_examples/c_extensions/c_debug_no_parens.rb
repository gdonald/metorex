# A C extension opening the debug inspector and reading the self, class,
# binding and location of each frame on the call stack.
require "tmpdir"
require_relative "build_helper"

class Reporter
  def report_frames(debug)
    marker = "inside report_frames"
    debug.frames
  end
end

def describe_frame frame
  receiver, klass, binding, iseq, location = frame
  locals = binding ? binding.local_variables.sort & [:debug, :marker, :outer] : nil
  method = binding ? binding.eval("__method__") : nil
  [receiver.class, klass, locals, method, iseq, location.label, location.lineno]
end

directory = Dir.mktmpdir
require build_extension("c_debug.c", "c_debug", directory)
debug = CDebug.new

outer = "at the top"
Reporter.new.report_frames(debug).each { |frame| p describe_frame(frame) }

first = debug.frames.first
p [first[0].equal?(debug), first[1], first[2]]
report { debug.frame_at 100 }
report { debug.frame_at(-1) }
p debug.frame_at(0).equal? debug

reporter_frame = Reporter.new.report_frames(debug)[1]
p reporter_frame[2].local_variable_get :marker
p reporter_frame[2].receiver.class

p CDebug::LOADED_FROM.first.values_at(1, 4).map { |held| held.is_a?(Thread::Backtrace::Location) ? held.label : held }

FileUtils.rm_rf directory
