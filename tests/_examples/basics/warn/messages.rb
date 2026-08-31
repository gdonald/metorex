# Kernel#warn writes through $stderr, honoring $VERBOSE and its keywords.

class Recorder
  attr_reader :lines

  def initialize
    @lines = []
  end

  def write(text)
    @lines.push text
  end
end

recorder = Recorder.new
$stderr = recorder

$VERBOSE = nil
warn "silenced"

$VERBOSE = true
warn "plain"
warn "already ended\n"
warn "first", "second"
warn ["from", "an array"]
warn "categorized", category: :deprecated

empty = {}
warn(**empty)
warn("with empty keywords", **empty)

def outer(message, level)
  inner message, level
end

def inner(message, level)
  warn message, uplevel: level
end

outer "too far", 100

begin
  warn "bad", category: Object.new
rescue TypeError
  recorder.write "TypeError for an unconvertible category\n"
end

begin
  warn "bad", uplevel: -1
rescue ArgumentError
  recorder.write "ArgumentError for a negative uplevel\n"
end

begin
  warn "bad", uplevel: "one"
rescue TypeError
  recorder.write "TypeError for a non-Integer uplevel\n"
end

$stderr = STDERR
print recorder.lines.join
