class Capture
  def write(text)
    puts "captured: #{text}"
  end
end

$stderr = Capture.new

abort "redirected message"

puts "never reached"
