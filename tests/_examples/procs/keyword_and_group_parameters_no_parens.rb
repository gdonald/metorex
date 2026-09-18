# `|**nil|` says a block takes no keyword arguments at all, whether it is a
# lambda or a proc.
refuses_keywords = proc { |**nil| :ok }
puts refuses_keywords.call.inspect
begin
  refuses_keywords.call a: 1
rescue ArgumentError => refused
  puts refused.message
end

# A keyword declared without a default has to be given.
needs_a_keyword = proc { |a:, **rest| [a, rest] }
begin
  needs_a_keyword.call
rescue ArgumentError => refused
  puts refused.message
end

# A block whose only positional place is a splat keeps a lone array whole.
gathers = proc { |*taken, **keywords| [taken, keywords] }
puts gathers.call([1, { a: 1 }]).inspect

# A group asks anything but an Array for `to_ary`, and refuses what answers
# something else.
class Pair
  def to_ary
    [1, 2]
  end
end

class NotAPair
  def to_ary
    1
  end
end

spreads = proc { |(first, second)| [first, second] }
puts spreads.call(Pair.new).inspect
begin
  spreads.call NotAPair.new
rescue TypeError => refused
  puts refused.class
end
