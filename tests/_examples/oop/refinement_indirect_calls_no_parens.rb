# A refinement in force covers a call written out and one reached through
# `send`, a Symbol turned into a block, or text being built.
class Refined
  def foo
    "plain"
  end

  def to_s
    "plain"
  end
end

refinement = Module.new do
  refine Refined do
    def foo
      "refined"
    end

    def to_s
      "refined"
    end
  end
end

held = []
Module.new do
  using refinement
  subject = Refined.new
  held << subject.foo
  held << subject.send(:foo)
  held << subject.__send__(:foo)
  held << subject.public_send(:foo)
  held << [Refined.new].map(&:foo).first
  held << "#{subject}"
end

puts held.inspect
puts Refined.new.foo

# The same name asked about rather than called.
Module.new do
  using refinement
  subject = Refined.new
  puts subject.respond_to? :foo
  puts subject.method(:foo).call
  puts subject.method(:foo).class == Method
end

# A refinement on a module reaches what takes that module in, a method the
# object's own class writes stands ahead of a refinement on an ancestor, and
# a module's own refinement stands ahead of one it includes.
module Countable
  def described
    "plain"
  end
end

class Holder
  include Countable
end

class Narrower < Holder
  def described
    "from the subclass"
  end
end

on_module = Module.new do
  refine Countable do
    def described
      "from the module refinement"
    end
  end
end

Module.new do
  using on_module
  puts Holder.new.described
  puts Narrower.new.described
end
