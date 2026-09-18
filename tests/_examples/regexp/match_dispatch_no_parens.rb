# A string matched against another string refuses outright, and one matched
# against anything else hands the match to that object.
begin
  "some string" =~ "another string"
rescue TypeError => refused
  puts refused.message
end

class Matcher
  def =~(other)
    "matched #{other}"
  end
end
puts "w00t" =~ Matcher.new

# `match` reads a pattern written as anything that answers to_str.
class Pattern
  def to_str
    "l+"
  end
end
puts "hello".match(Pattern.new)[0]

# A pattern carrying a `match` of its own answers for the string.
pattern = /./.dup
def pattern.match subject
  "asked #{subject}"
end
puts "hello".match(pattern)
puts pattern.match "hello"
