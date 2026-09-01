# A call that visibility refuses still reaches a user-defined
# `method_missing`, and the NoMethodError it raises names the method and the
# object the call was made on.

class Guarded
  def method_missing(name, *args)
    "handled #{name} with #{args.inspect}"
  end

  def hidden
    :never
  end
  private(:hidden)

  def shielded
    :never
  end
  protected(:shielded)
end

guarded = Guarded.new
puts(guarded.hidden)
puts(guarded.shielded(1, 2))
puts(guarded.absent(:arg))

class Plain
  def hidden
    :never
  end
  private(:hidden)
end

plain = Plain.new
begin
  plain.hidden
rescue NoMethodError => error
  puts(error.message)
  puts(error.name.inspect)
  puts(error.receiver.equal?(plain).to_s)
end

# `super` from an override reaches the default, which raises.
class Passthrough
  def method_missing(name, *args)
    super
  end
end

begin
  Passthrough.new.absent
rescue NoMethodError => error
  puts(error.message)
  puts(error.name.inspect)
end
