# A proc's binding answers the `self` of the place the proc was written.
handler = proc { |id| id }
p(handler.binding.receiver)
class Holder
  def make
    proc { }
  end
end
holder = Holder.new
p(holder.make().binding.receiver.equal?(holder))
p(handler.dup() == handler)
