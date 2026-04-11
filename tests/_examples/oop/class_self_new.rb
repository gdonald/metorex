class Foo
  def initialize
    @x = 42
    puts "init called, @x=#{@x}"
  end

  def self.make
    puts "make: self=#{self}"
    inst = new
    puts "make: inst.class=#{inst.class}"
    inst
  end
end

f = Foo.make
puts "f.class=#{f.class}"
