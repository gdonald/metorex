# A superclass can be named from the top level with a leading `::`, and a
# `super` call can carry a block of its own.

class Ledger
  def initialize
    @entries = []
  end

  def record
    @entries << yield if block_given?
    @entries
  end
end

module Accounting
  # The nested name is reached from the top level rather than from the
  # module that is being opened.
  class Ledger < ::Ledger
    def record
      super { :nested }
    end
  end
end

module Bookkeeping
  class Ledger < ::Ledger
    def record
      super() { :explicit }
    end

    def initialize
      super
      @opened = true
    end

    def opened?
      @opened
    end
  end
end

p(Accounting::Ledger.new.record)
p(Bookkeeping::Ledger.new.record)
p(Bookkeeping::Ledger.new.opened?)

module Archiving
  # The same block, written the other way round.
  class Ledger < ::Ledger
    def record
      super do
        :archived
      end
    end
  end
end

p(Archiving::Ledger.new.record)

# A block on `super` reaches the parent even when the caller gave one, and
# bare `super` still forwards the enclosing method's arguments.
class Tally
  def add(amount)
    amount + yield
  end
end

class DoubleTally < Tally
  def add(amount)
    super { 10 }
  end
end

p(DoubleTally.new.add(5))
