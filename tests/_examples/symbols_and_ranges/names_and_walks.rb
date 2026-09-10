p :glark.upcase
p :GLARK.downcase
p :hello_world.capitalize
p :AbC.swapcase
p :a.succ
p :Z.next
p :ruby.name
p :ruby.id2name
p :ruby.intern
p :ruby.length
p Symbol.include?(Comparable)
p((:a <=> :b))

built = :"a#{1 + 1}b"
p built
p built.class.to_s

p "\u{E0}Bc"
p "àbc".to_sym
p "\xC3\x9Cber"
p "\xC3\x9Cber".length
p "a\sb"
p "\u{48 49}"
p 65.chr
p 0x3042.chr("UTF-8")

p "az".succ
p "zz".succ
p "a9".succ
p "1.9".succ
p "hello world".capitalize
p "AbC".swapcase
p "123.45e1".to_f
p "1_234.5".to_f
p "_5".to_f
p "5e".to_f

p (1..5).to_a
p ('A'..'D').to_a
p ('D'..'A').to_a
p ('ax'..'bd').to_a
p (:A..:D).to_a
p (1...4).to_s
p (1...4).inspect
p (1..).inspect
p (..3).inspect
p ((1..3) == (1..3))
p ((1..3).eql?(1..3))
p ((1..3) == (1...3))
p (1..3).count
p (1..).count

walked = []
('a'..'c').each { |letter| walked << letter }
p walked

begin
  (1..).to_a
rescue RangeError => error
  puts error.message
end

p Range.new(1, 3).to_a
p Range.new(1, 3, true).to_a
