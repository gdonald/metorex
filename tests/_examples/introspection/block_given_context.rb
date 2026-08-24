module Reporter
  def self.direct
    block_given?
  end

  def self.with_named_block(&handler)
    block_given?
  end

  def self.inside_block
    yield_self { block_given? }
  end

  def self.via_kernel
    Kernel.block_given?
  end

  def self.via_send
    self.send :block_given?
  end

  class << self
    define_method(:from_define_method) do
      block_given?
    end
  end
end

puts Reporter.direct { }
puts Reporter.direct
puts Reporter.with_named_block { }
puts Reporter.with_named_block
puts Reporter.inside_block { }
puts Reporter.inside_block
puts Reporter.via_kernel { }
puts Reporter.via_kernel
puts Reporter.via_send { }
puts Reporter.via_send
puts Reporter.from_define_method { }
puts block_given?
