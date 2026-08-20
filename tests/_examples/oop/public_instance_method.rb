module Base
  def open_method
  end

  protected

  def guarded
  end

  private

  def hidden
  end
end

module Layer
  include Base

  def layer_method
  end
end

from_base = Layer.public_instance_method(:open_method)
puts from_base.owner.inspect
puts (from_base.owner == Base).inspect

from_layer = Layer.public_instance_method(:layer_method)
puts (from_layer.owner == Layer).inspect
puts (Base.public_instance_method("open_method") == Base.public_instance_method(:open_method)).inspect

[:guarded, :hidden, :missing].each do |name|
  begin
    Base.public_instance_method(name)
  rescue NameError => error
    puts "#{name}: #{error.name.inspect}"
  end
end

begin
  Module.new.public_instance_method(nil)
rescue TypeError => error
  puts error.message
end

puts Module.new.method(:public_instance_method).arity.inspect
