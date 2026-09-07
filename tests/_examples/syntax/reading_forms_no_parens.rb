# Forms the parser reads: a control-flow keyword used for its value, an empty
# pair of parentheses, a group of statements, percent literals, a call written
# with a leading dot, and a call that steps over a nil receiver.
case 1
when 1 then :matched
end.to_s.tap { |shown| p shown }

unless false then
  :kept
end.to_s.tap { |shown| p shown }

counted = 0
while counted < 3
  counted += 1
end.inspect.tap { |shown| p shown }

p [0, (), 2]
empty_keys = {() => ()}
p empty_keys
grouped = (1; 2; 3)
p grouped

names = 3
filled_words = %W(a #{names} c)
filled_symbols = %I(a b#{names})
plain_words = %w(x y)
plain_symbols = %i(x y)
raw_text = %q{a#{1}b}
escaped_text = %q(a\(b)

p filled_words
p filled_symbols
p plain_words
p plain_symbols
p raw_text
p escaped_text

p [1, 2, 3]
  .map { |number| number * 2 }
  .select { |number| number > 2 }

p nil&.length
p "hello"&.length
p nil&.map { |item| item }
p [1, 2]&.map { |item| item + 1 }
