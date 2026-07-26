puts Class.new { define_method(:foo) { return 42 } }.new.foo.inspect
puts Class.new { define_method(:foo) { break 42 } }.new.foo.inspect
puts Class.new { define_method(:foo) { next 42 } }.new.foo.inspect

retrying = Class.new do
  seen = []
  define_method(:foo) do
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

# A trailing comma makes a Proc destructure a lone array argument; the method
# built from the same block keeps the plain single-parameter arity.
puts proc { |first,| first }.call([1, 2]).inspect
puts Class.new { define_method(:m) { |first,| first } }.new.m([1, 2]).inspect
