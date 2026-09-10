# A log with a severity on each message, a monitor that guards a section of
# code, and a scratch file that is removed when it is closed.

require 'logger'
require 'monitor'
require 'tempfile'

path = File.join(Dir.tmpdir, "metorex_logging_#{Process.pid}.log")
File.delete(path) if File.exist?(path)

log = Logger.new(path)
log.level = Logger::WARN
p(log.level)
p(log.warn?)
p(log.debug?)

log.add(Logger::WARN, "written")
log.add(Logger::DEBUG, "below the level")
log.close

lines = File.readlines(path)
p(lines.length)
p(lines.last.include?("WARN -- : written"))
File.delete(path)

# A severity may be named rather than numbered.
named = Logger.new(path, level: :error)
p(named.level)
named.close
File.delete(path)

# A monitor may be entered more than once by the same thread, and refuses to
# be left by anyone who is not holding it.
guard = Monitor.new
p(guard.mon_locked?)
guard.synchronize do
  guard.synchronize do
    p(guard.mon_locked?)
    p(guard.mon_owned?)
  end
end
p(guard.mon_locked?)

begin
  guard.exit
rescue ThreadError => problem
  p(problem.class)
end

# A scratch file has a name nobody else is using.
scratch = Tempfile.new("example")
p(File.exist?(scratch.path))
scratch.puts("held")
scratch.close
p(File.exist?(scratch.path))
scratch.unlink
p(scratch.path)
