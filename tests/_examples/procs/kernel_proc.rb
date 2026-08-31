plain = proc { 42 }
puts plain.lambda?
puts plain.call

stabby = -> { 7 }
wrapped = proc(&stabby)
puts wrapped.equal?(stabby)
puts wrapped.lambda?

class Holder
  def build
    send(:proc) { :from_send }
  end

  def no_block
    proc
  end

  def escapes
    proc { return :early }.call
    :never_reached
  end
end

holder = Holder.new
puts holder.build.call.inspect
puts holder.escapes.inspect

begin
  holder.no_block
rescue ArgumentError => error
  puts "#{error.class}: #{error.message}"
end

self_referential = nil
self_referential = proc { self_referential }
puts self_referential.equal?(self_referential)
puts self_referential.equal?(plain)

puts Kernel.private_instance_methods(false).include?(:proc)
