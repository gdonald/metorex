breaker = Class.new do
  define_method :foo do
    break 42
  end
end
puts breaker.new.foo.inspect

skipper = Class.new do
  define_method :foo do
    next 42
  end
end
puts skipper.new.foo.inspect

retrying = Class.new do
  seen = []
  define_method :foo do
    if seen.empty?
      seen << :first
      redo
    else
      seen << :second
      seen
    end
  end
end
puts retrying.new.foo.inspect
