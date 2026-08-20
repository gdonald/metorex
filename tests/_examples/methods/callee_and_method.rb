class Reporter
  def plain
    [__callee__, __method__]
  end

  alias_method :aliased, :plain

  def in_block
    (1..2).map { __callee__ }
  end

  define_method(:defined) do
    __callee__
  end

  def from_send
    send "__callee__"
  end

  @@during_body = __callee__

  def during_body
    @@during_body
  end
end

reporter = Reporter.new
puts reporter.plain.inspect
puts reporter.aliased.inspect
puts reporter.in_block.inspect
puts reporter.defined.inspect
puts reporter.from_send.inspect
puts reporter.during_body.inspect
puts __callee__.inspect
puts __method__.inspect

outside = proc { __callee__ }
puts outside.call.inspect

class Super
  def greet
    "super"
  end
end

class Sub < Super
  def greet
    super + "-sub"
  end

  alias_method :hello, :greet
end

puts Sub.new.hello
