# `Exception#exception` answers self with no argument, and otherwise a copy
# carrying the new message. The copy is not re-initialized, so a subclass
# keeps the state it set. `Exception.exception` is another name for `new`.

original = RuntimeError.new("first")
puts(original.equal?(original.exception).to_s)
puts(original.equal?(original.exception(original)).to_s)

renamed = original.exception("second")
puts(renamed.class.to_s)
puts(renamed.message)
puts(original.message)
puts(renamed.equal?(original).to_s)

class Tagged < StandardError
  attr_reader :tag

  def initialize(tag)
    @tag = tag
  end
end

tagged = Tagged.new(:boom)
copy = tagged.exception("message")
puts(copy.class.to_s)
puts(copy.tag.inspect)
puts(copy.message)

# The class-level form builds one the way `new` does.
puts(Exception.exception("built").message)
puts(Exception.exception.message)
puts(RuntimeError.exception.class.to_s)

# An exception built with no message reports its class name; one built with an
# empty message reports that.
puts(RuntimeError.new.message)
puts(RuntimeError.new("").message.inspect)
