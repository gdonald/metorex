# A parameter written with no name of its own reports the mark it was written
# with, and `**nil` says the method takes no keyword at all.
def anonymous_splat(*); end
def anonymous_keyrest(**); end
def anonymous_block(&); end
def forwarded(...); end
def refuses_keywords(**nil); end
p method(:anonymous_splat).parameters
p method(:anonymous_keyrest).parameters
p method(:anonymous_block).parameters
p method(:forwarded).parameters
p method(:refuses_keywords).parameters

# A method the interpreter answers itself reports the kinds of its parameters
# and no names, since there is no source to have named them.
p "foo".method(:+).parameters
p [].method(:pop).parameters
p "foo".method(:delete!).parameters

class Holder
  attr_writer :held
end
p Holder.instance_method(:held=).parameters

# A subclass of Proc stands for the block it was built with, and its own
# `initialize` runs with the block already attached.
class Counted < Proc
  attr_reader :made
  def initialize
    @made = true
    super
  end
end
counted = Counted.new { "hello" }
p counted.class == Counted
p counted.call
p counted.made

# A callable handed over with `&` is answered as it stands.
p Counted.new(&counted).equal?(counted)
