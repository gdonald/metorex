class Capture
  def initialize
    @written = ""
  end

  def write text
    @written += text.to_s
  end

  # Ruby hands `puts` to the stream itself, so a stand-in stream answers it.
  def puts *values
    return write "\n" if values.empty?
    values.each { |value| write value.to_s + "\n" }
    nil
  end

  def written
    @written
  end
end

capture = Capture.new
original = $stdout
$stdout = capture

puts "through puts"
print "through print"
p "through p"
puts

$stdout = original
written = capture.written
puts written.inspect

puts :symbol
print :symbol
puts
p :symbol

$_ = "last line"
print
puts

class Speaker
  def to_s
    "speaker to_s"
  end

  def describe
    to_s
  end
end

described = Speaker.new.describe
puts described
