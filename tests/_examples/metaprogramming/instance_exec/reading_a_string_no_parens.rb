# The same run written without parentheses where they can be left off.
#
# `instance_eval` handed a String runs it against the receiver: the code reads
# constants from the receiver's singleton class and class before the scopes
# the caller was written in, it sees the caller's own locals, and the file and
# line it is counted from are the ones it was given.

module ReceiverScope
  FOO = :receiver_scope

  class Parent
    FOO = :parent
  end

  class Receiver < Parent
    FOO = :receiver

    def initialize
      singleton_class.const_set(:FOO, :singleton_class)
    end
  end

  class PlainReceiver < Parent
  end
end

module CallerScope
  FOO = :caller_scope

  class Caller
    FOO = :caller

    def read(receiver)
      receiver.instance_eval "FOO"
    end
  end
end

p CallerScope::Caller.new.read ReceiverScope::Receiver.new
p CallerScope::Caller.new.read ReceiverScope::PlainReceiver.new

# The code sees the caller's locals, and a name it assigns is the caller's own.
held = nil
Object.new.instance_eval "held = :assigned"
p held

# The file and the line it is counted from are the ones it was given.
refused = begin
  Object.new.instance_eval "raise 'from the string'", "a_file", 10
rescue RuntimeError => held
  held
end
p refused.backtrace.first

# With none given, the code is counted as written where the call was made.
where = Object.new.instance_eval "[__FILE__, __LINE__]"
p where[0].start_with? "(eval at "
p where[1]

# A block keeps the scopes it was written in.
class BlockScope
  @@noted = :block_scope

  def block
    -> * { @@noted }
  end
end

class OtherCaller
  @@noted = :other_caller

  def read(receiver, block)
    receiver.instance_eval(&block)
  end
end

p OtherCaller.new.read(Object.new, BlockScope.new.block)
