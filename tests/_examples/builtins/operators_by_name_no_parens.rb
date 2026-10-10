# An operator called by name raises what Ruby raises for the pair: a number
# asks the other side to coerce itself, a collection wants one of its own
# kind or an Integer, a value with no such operator has no such method, and
# a comparison that fails names both sides.
def outcome
  yield.inspect
rescue StandardError => error
  "#{error.class}: #{error.message.gsub(/0x\h+/, "X")}"
end

cases = [
  [1, :+, nil], [1, :<<, nil], [1, :&, true], [1, :[], nil], [1.5, :&, true],
  [1.5, :<, :sym], [Rational(1, 2), :+, :sym], [Rational(1, 2), :<, "s"],
  [Rational(1, 2), :|, true], [Complex(1, 2), :+, "s"], [Complex(1, 2), :**, Object.new],
  [Complex(1, 2), :<, 1], [Complex(1, 2), :%, 2], ["s", :+, nil], ["s", :*, nil],
  ["s", :*, "s"], ["s", :[], :sym], ["s", :<<, true], ["s", :=~, "t"], ["s", :=~, [1]],
  ["s", :&, true], [[1], :+, nil], [[1], :-, "s"], [[1], :&, true], [[1], :*, nil],
  [[1], :^, true], [{a: 1}, :<, nil], [{a: 1}, :+, 1], [:sym, :<, :sym], [:sym, :<, 1],
  [:sym, :-@, nil], [:sym, :[], nil], [nil, :+, 1], [true, :<, 1], [Object.new, :-, 1],
  [(1..2), :+, 1], [Time.at(0).utc, :-, "s"], [1, :=~, 1]
]
cases.each do |receiver, operator, argument|
  shown = operator == :-@ ? outcome { receiver.public_send(operator) } : outcome { receiver.public_send(operator, argument) }
  puts "#{receiver.class} #{operator}: #{shown}"
end
p([1, 2.5, "s", [1], nil, :sym, Object.new].map { |value| value.public_send(:!) })
