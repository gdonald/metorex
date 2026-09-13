spreadable = Object.new
def spreadable.to_a
  [2, 3, 4]
end
puts([1, *spreadable].inspect)

plain = Object.new
puts([1, *plain].class.to_s)

def collect(*given)
  given
end
puts(collect(1, *spreadable).inspect)

letters = ("a".."e").to_a
bounds = [1, 3]
letters[*bounds] = "x"
puts(letters.inspect)

def hands_over(values)
  yield(*values)
end
puts(hands_over(nil) { |*taken| taken }.inspect)
puts(hands_over([1, 2]) { |*taken| taken }.inspect)

puts(%W(a\  b\tc).inspect)
puts(%w(a b\ c).inspect)
