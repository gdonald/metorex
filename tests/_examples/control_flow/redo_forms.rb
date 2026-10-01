# Where `redo` restarts: the body of the closest block or loop, without
# asking the iterator for its next value, and after any ensure clause runs.

calls = []
-> {
  calls << 1
  redo if calls.size < 2
  calls << 2
}.call
p(calls)

pending = [2, 3]
processed = []
[1, 2, 3, 4].each do |value|
  begin
    processed << value
    raise StandardError, "included" if pending.include?(value)
  rescue StandardError
    pending.delete(value)
    redo
  end
end
p(processed)

list = []
[1, 2, 3].each do |value|
  list << value
  break if list.size == 6
  redo if value == 3
end
p(list)

list = []
[1, 2, 3].each do |value|
  list << value
  begin
    list << 10 * value
    redo if list.count(1) == 1
  ensure
    list << 100 * value
  end
end
p(list)

begin
  eval("def restarts; redo; end")
rescue SyntaxError
  p(SyntaxError)
end
