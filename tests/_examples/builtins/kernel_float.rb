puts Float(1)
puts Float(1.5)
puts Float("10")
puts Float("10.0")
puts Float(" +10 ")
puts Float("-10")
puts Float("1_000")
puts Float("1.")
puts Float("2e3")
puts Float("2e-3")
puts Float("0x10")
puts Float("-0x7b")
puts Float("0x0.8")
puts Float("0x1p10")
puts Float(Complex(1))

puts Float("2e1000")
puts Float("2e-1000")

nan = Float("0") / Float("0")
puts nan.nan?
puts Float(nan).nan?
puts Float("2e1000").infinite?
puts 1.5.finite?

class Measured
  def to_f
    1.25
  end
end

puts Float(Measured.new)

["float", "10.0.0", "10D", "1+1", "_1", "10_", " ", "1 2", "2e", "e2", "0x_10"].each do |text|
  begin
    Float(text)
    puts "no error for #{text}"
  rescue ArgumentError => error
    puts error.message
  end
end

begin
  Float(nil)
rescue TypeError => error
  puts error.message
end

begin
  Float(Complex(2, 3))
rescue RangeError => error
  puts error.message
end

p Float("abc", exception: false)
p Float(nil, exception: false)
puts Kernel.private_instance_methods.include?(:Float)
