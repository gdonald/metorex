# An exception handed to a sleeping thread is raised where it sleeps, so its
# backtrace starts at `Kernel#sleep`, and a thread's block is named by how
# many blocks deep it was written.
def short(lines)
  lines.map() { |line| line.sub(/\A.*\//, "") }
end

[1].each() do
  sleeper = Thread.new() do
    Thread.current().report_on_exception = false
    sleep()
  end
  Thread.pass() until sleeper.stop?()
  sleeper.raise("wake up")
  begin
    sleeper.join()
  rescue => woken
    p(short(woken.backtrace()))
  end
end

# A backtrace the exception was given, or one it was raised with before, is
# the one it keeps.
given = StandardError.new("given")
given.set_backtrace(["somewhere:1"])
begin
  raise(given)
rescue => kept
  p(kept.backtrace())
end

# `\#` in a regex stays an escaped `#`, interpolation or not, so extended
# mode reads it as a character rather than the start of a comment.
number = 1
p(/a\##{number}/x.source())
p(/a\##{number}/x.match?("a#1"))

# The file a fiber's code runs from is its own, so `__FILE__` reads the same
# after a thread written elsewhere has run.
before = __FILE__
Thread.new() { :other }.join()
p(__FILE__ == before)
