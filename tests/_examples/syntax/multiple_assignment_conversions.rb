# How a multiple assignment takes its right-hand side apart: `to_ary` for a
# lone value, `to_a` for a splat, an Array subclass as the Array it is, and
# the errors when a conversion answers something else. Also where a local
# begins, and the notice for reading a global nothing set.

pair = Object.new
def pair.to_ary
  [1, 2]
end
a, b, c = pair
p([a, b, c])
first, *rest = pair
p([first, rest])

hidden = Object.new
class << hidden
  private

  def to_ary
    [3, 4]
  end
end
a, b = hidden
p([a, b])

declines = Object.new
def declines.respond_to?(name, include_private = false)
  name == :to_ary ? false : super
end
def declines.to_ary
  [:never]
end
a, b = declines
p([a.equal?(declines), b])

empty = Object.new
def empty.to_ary
  nil
end
a, b = empty
p([a.equal?(empty), b])

wrong = Object.new
def wrong.to_ary
  1
end
begin
  a, b = wrong
rescue TypeError => error
  p(error.message)
end

listed = Class.new(Array)[5, 6]
a, b = listed
p([a, b])

counted = Object.new
def counted.to_a
  [7, 8].freeze
end
a, b = 1, *counted
p([a, b])
spread = *counted
p([spread, spread.frozen?])

source = [1, 2]
copied = *source
p(copied.equal?(source))

refuses = Object.new
def refuses.to_a
  1
end
begin
  single = *refuses
rescue TypeError => error
  p(error.message)
end

a, (b, c), d = 1, pair, 4
p([a, b, c, d])

if false
  later = 1
end
1.times { p([defined?(later), later]) }

[1].each do
  value = :inside
end
def value(given)
  given
end
p(value [])

warned = []
collector = Object.new
collector.define_singleton_method(:write) { |text| warned << text }
old_stderr = $stderr
$stderr = collector
old_verbose = $VERBOSE
$VERBOSE = true
read = $never_assigned_global
$lazily_assigned ||= 1
$VERBOSE = old_verbose
$stderr = old_stderr
p(warned.map { |text| text.sub(/\A.*warning: /, "") })
p($FILENAME)
