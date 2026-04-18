class Pair
  def initialize(v)
    @v = v
  end

  def <=>(other)
    @v <=> other.instance_variable_get(:@v)
  end
end

a = Pair.new(5)
b = Pair.new(10)

a.singleton_class.class_eval do
  define_method(:<=>) { |other| 1 }
end

puts((a > b).inspect)
puts(a.send(:<=>, b))
