Target = Class.new do
  def describe
    "plain"
  end
end

Refinement = Module.new do
  refine Target do
    def describe
      "refined"
    end
  end
end

returned = nil
host = Module.new do
  returned = using(Refinement)
end
puts((returned == host).inspect)

Module.new do
  def self.defined_before(object)
    object.describe
  end

  using(Refinement)

  puts(defined_before(Target.new).inspect)

  def self.defined_after(object)
    object.describe
  end

  puts(defined_after(Target.new).inspect)
  puts(Target.new.describe.inspect)
end

puts(Target.new.describe.inspect)
