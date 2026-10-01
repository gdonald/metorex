# What `super` reaches and what it hands on: a bare `super` passes the
# parameters as they stand, a block reaches the method it was written in, an
# alias starts past the class that wrote the body, and a method nothing above
# defines goes to method_missing.

class Recorder
  def record *values, **options
    [values, options]
  end

  def pair first, second
    [first, second]
  end
end

class Adjusted < Recorder
  def record first, second = 2, *rest, last, flag: :off
    first = 10
    rest << :added
    super
  end

  def pair _, _
    _ = :changed
    super
  end
end
p Adjusted.new.record 1, 5
p Adjusted.new.record 1, 3, 4, 5, flag: :on
p Adjusted.new.pair :a, :b

class InBlock < Recorder
  def record value
    [1].map { super() }.first
  end

  def pair first, second
    -> { super }.call
  end
end
p InBlock.new.record 7
p InBlock.new.pair 3, 4

built = Class.new Recorder do
  define_method :pair do |first, second|
    super
  end
end
begin
  built.new.pair 1, 2
rescue RuntimeError => error
  p error.message
end

class Hidden
  undef_method :frozen?
end

class Asks < Hidden
  def frozen?
    super
  end

  def method_missing name, *arguments
    [:missing, name]
  end
end
p Asks.new.frozen?

class Named
  def label
    [:named]
  end
end

class Relabeled < Named
  def label
    [:relabeled] + super
  end
end

class Copied < Relabeled
  alias_method :tag, :label
end
p Copied.new.tag

module Plain
  def build
    :plain
  end
end

module Wrapped
  def build
    [:wrapped, super]
  end
end

class Factory
  extend Plain
  extend Wrapped
end
p Factory.build

module Sends
  def __send__ name, *arguments
    super
  end
end

class Sender
  include Sends

  def answer
    42
  end
end
p Sender.new.__send__ :answer
