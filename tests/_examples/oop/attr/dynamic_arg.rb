class StringLike
  def to_str; "test"; end
end

o = StringLike.new
c = Class.new { attr o }
inst = c.new
inst.instance_variable_set(:@test, "hello")
puts inst.respond_to?("test")
puts inst.test
