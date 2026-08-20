module Helpers
  def named_form
    :named
  end

  def other
    :other
  end

  module_function :named_form

  module_function

  def toggled
    :toggled
  end

  public

  def after_public
    :after_public
  end
end

puts Helpers.named_form.inspect
puts Helpers.toggled.inspect
puts Helpers.respond_to?(:other).inspect
puts Helpers.respond_to?(:after_public).inspect
puts Helpers.private_instance_methods(false).sort.inspect
puts Helpers.public_methods.include?(:named_form).inspect

class Consumer
  include Helpers

  def call_it
    named_form
  end
end

puts Consumer.new.call_it.inspect
begin
  Consumer.new.named_form
rescue NoMethodError
  puts "NoMethodError"
end

module Base
  def label
    "base"
  end
end

Layered = Module.new do
  extend Base
  module_function

  def label
    ["layered", super]
  end
end

puts Layered.label.inspect
puts Module.private_instance_methods.include?(:module_function).inspect
