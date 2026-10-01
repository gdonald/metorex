positions = [[2, 10], [1, 12], [1, 3], [2, 1]]
p(positions.sort_by { |line, column| [line, column] })
p(positions.min_by { |position| position })
p(positions.max_by { |position| position })

Token = Struct.new(:pos, :text)
tokens = [Token.new([1, 12], "end"), Token.new([1, 3], "def"), Token.new([1, 0], "x")]
p(tokens.sort_by(&:pos).map(&:text))

held = positions.dup
held.sort_by! { |position| position }
p(held)

begin
  [[1], ["a"]].sort_by { |key| key }
rescue ArgumentError => e
  p(e.message)
end
