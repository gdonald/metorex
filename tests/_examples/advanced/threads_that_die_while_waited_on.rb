# A thread that dies of an exception hands it to whoever waits on it, even
# when another thread holds the turn while the death is being reported.
busy = Thread.new { loop { 200.times { |number| number * number } } }
seen = Hash.new(0)
20.times do
  dying = Thread.new do
    Thread.current.report_on_exception = false
    begin
      1 / 0
    rescue ZeroDivisionError
      Thread.current.raise
    end
  end
  begin
    seen[[:value, dying.value]] += 1
  rescue RuntimeError => error
    seen[[error.class, error.message]] += 1
  end
end
busy.kill
p(seen)
