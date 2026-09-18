# Every required parameter is filled before any optional one, whether it
# stands before the splat or after it.
def around(first, second = 9, *rest, last)
  [first, second, rest, last]
end
puts around(1, 2).inspect
puts around(1, 2, 3).inspect
puts around(1, 2, 3, 4).inspect

# The names after a splat in a destructured group take the last values, and
# where there are too few they run out into nil.
def spread((a, b, *c, d, e))
  [a, b, c, d, e]
end
puts spread([1, 2, 3]).inspect
puts spread([1, 2, 3, 4]).inspect
puts spread([1, 2, 3, 4, 5]).inspect

# A splat among the subscripts of an indexed write spreads across them.
class Recorder
  attr_reader :taken

  def []=(*args)
    @taken = args
  end
end

recorder = Recorder.new
middle = [2, 3]
recorder[1, *middle] = 4
puts recorder.taken.inspect

# A block written alongside a block argument is refused.
begin
  eval "[1].each(&:to_s) { 42 }"
rescue SyntaxError
  puts "SyntaxError"
end

# An object that answers no to_proc is no block at all.
begin
  [1].each(&Object.new)
rescue TypeError => refused
  puts refused.message
end
