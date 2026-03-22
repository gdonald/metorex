# Backtrace method example

class ErrorDemo
  def outer
    self.middle
  end

  def middle
    self.inner
  end

  def inner
    raise("Error in inner method")
  end
end

begin
  demo = ErrorDemo.new
  demo.outer
rescue => e
  puts "Caught: #{e.message}"

  trace = e.backtrace
  puts "Backtrace array length: #{trace.length}"

  # Print first few frames
  if trace.length > 0
    puts "First frame: #{trace[0]}"
  end
end
