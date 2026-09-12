# A copy of a callable is its own object that still compares equal, and
# `public_send` reaches only what a caller outside the object could write.
class Holder
  def open_door
    :open
  end

  def shut_door
    :shut
  end
  protected :shut_door

  def hidden_door
    :hidden
  end
  private :hidden_door
end

held = Holder.new
taken = held.method(:open_door)
copied = taken.clone

p(copied == taken)
p(copied.equal?(taken))
p(copied.call)

taken.instance_variable_set(:@note, :kept)
p(taken.clone.instance_variables)

step = proc { :stepped }
p(step.dup == step)
p(step.dup.equal?(step))

p(held.public_send(:open_door))

[:shut_door, :hidden_door].each do |named|
  begin
    held.public_send(named)
  rescue NoMethodError => refused
    puts(refused.message)
  end
end

p(:upcase.to_proc.lambda?)
p(:upcase.to_proc.parameters)
p(:upcase.to_proc.call("shout"))
