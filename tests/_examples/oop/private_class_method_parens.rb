class Parent
  def self.inherited_secret
    :secret
  end
  private_class_method(:inherited_secret)

  def self.first
    :first
  end
  def self.second
    :second
  end
end

class Child < Parent
end

Child.private_class_method(:first, :second)

begin
  Child.inherited_secret
rescue NoMethodError
  puts "inherited_secret hidden"
end

begin
  Child.first
rescue NoMethodError
  puts "first hidden"
end

begin
  Child.second
rescue NoMethodError
  puts "second hidden"
end

Child.public_class_method(:first)
puts(Child.first.inspect)

class Listed
  def self.only
    :only
  end
  private_class_method([:only])
end

begin
  Listed.only
rescue NoMethodError
  puts "only hidden"
end

begin
  Class.new do
    private_class_method(:absent)
  end
rescue NameError
  puts "NameError for a missing method"
end

class Instance
  def instance_level
  end
end

begin
  Instance.private_class_method(:instance_level)
rescue NameError
  puts "NameError for an instance method"
end
