guard = Mutex.new
signal = ConditionVariable.new
begin
  guard.synchronize { signal.wait(guard) }
rescue Exception => error
  p(error.class)
  puts(error.message.lines.first)
end
