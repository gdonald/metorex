Toggled = Module.new do
  send :private

  def hidden
  end

  send :protected

  def guarded
  end

  send :public

  def open
  end
end

puts Toggled.private_instance_methods(false).inspect
puts Toggled.protected_instance_methods(false).inspect
puts Toggled.public_instance_methods(false).inspect

Returned = Module.new do
  def first
  end
  def second
  end

  puts private(:first).inspect
  puts private([:first, :second]).inspect
  puts private(:first, :second).inspect
  puts public.inspect
end

Evaled = Module.new do
  eval "private\ndef in_eval; end"
end
puts Evaled.private_instance_methods(false).inspect

Closured = Module.new do
  1.times do
    send :private
  end

  def after_closure
  end
end
puts Closured.private_instance_methods(false).inspect

module Ancestor
  def shared
  end
  private :shared
end

module Descendant
  include Ancestor
  private :shared
end
puts Descendant.private_instance_methods(false).inspect

puts Module.private_instance_methods.include?(:private).inspect
