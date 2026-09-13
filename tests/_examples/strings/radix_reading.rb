# String#to_i reads the leading number in a base, taking a radix prefix that
# agrees with the base and lone underscores between digits. Reading stops at
# the first character the base does not name.
p "1_2_3asdf".to_i
p "_123".to_i
p "+0d56".to_i
p "0xFAZ".to_i(0)
p "0b112".to_i(0)
p "01778".to_i(0)
p "0b11".to_i(16)
p "-hello_world".to_i(32)
p ("z" * 24).to_i(36)
p "245789127594125924165923648312749312749327482".to_i

begin
  "".to_i(37)
rescue ArgumentError => trouble
  p trouble.message
end

# Integer#to_s writes the digits back out in the same base, however wide the
# number is.
p 255.to_s(16)
p (2 ** 80).to_s(36)
