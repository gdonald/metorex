# Where a `def` installs its method and what visibility it gets: a method
# defined at the top level is private on Object, `initialize` is always
# private, and a `def` run from a method body installs where that method was
# defined, publicly.
# Written with as few parentheses as Ruby allows.

def helper_at_top_level
  :helper
end
p Object.private_method_defined?(:helper_at_top_level)

class Account
  def initialize
    @opened = true
  end

  private

  def open_ledger
    def ledger_entry
      :entry
    end
  end
end
p Account.private_method_defined?(:initialize)
account = Account.new
account.send(:open_ledger)
p account.ledger_entry
p Account.public_method_defined?(:ledger_entry)

built = Class.new do
  def prepare
    def prepared
      :prepared
    end
  end
end
built.new.prepare
p built.new.prepared
p Object.new.respond_to?(:prepared)

holder = Object.new
holder.instance_eval do
  def configure
    def configured
      :configured
    end
  end
end
holder.configure
p holder.configured
p Object.new.respond_to?(:configured)

target = Object.new
def (target).label
  :labeled
end
p target.label

frozen = Class.new
frozen.freeze
begin
  frozen.class_eval do
    def refused; end
  end
rescue FrozenError => error
  p error.class
end

def needs_two(first, second = 2)
  [first, second]
end
begin
  needs_two
rescue ArgumentError => error
  p error.message
end

def reads_itself(value = value)
  value
end
p reads_itself

begin
  eval "def two_splats(*first, *second); end"
rescue SyntaxError
  p SyntaxError
end

class Runner
  def run(&block)
    Object.new.instance_exec(&block)
  end
end
Runner.new.run do
  [1].each do
    def defined_in_exec
      :ran
    end
  end
end
p Runner.method_defined?(:defined_in_exec)
