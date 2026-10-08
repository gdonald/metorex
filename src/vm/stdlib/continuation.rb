# `callcc` and the continuations it hands its block, which return from the
# block with the values they are called with.
warn "#{__FILE__}: warning: callcc is obsolete; use Fiber instead" unless $VERBOSE.nil?

class Continuation
  class << self
    undef_method :new
  end

  # Return from the `callcc` block this continuation came from, which answers
  # nothing for no values, the value for one, and an Array for several.
  def call(*values)
    unless @running
      raise NotImplementedError,
            "a continuation cannot be resumed once its callcc block has returned"
    end
    answer = case values.size
             when 0 then nil
             when 1 then values.first
             else values
             end
    throw @tag, answer
  end

  alias [] call

  def __start__(tag)
    @tag = tag
    @running = true
    self
  end

  def __finish__
    @running = false
  end
  private :__start__, :__finish__
end

module Kernel
  # Run the block with a continuation that returns from it at once, with
  # whatever it is called with.
  def callcc
    tag = Object.new
    continuation = Continuation.allocate.__send__(:__start__, tag)
    begin
      catch(tag) { yield continuation }
    ensure
      continuation.__send__(:__finish__)
    end
  end
  private :callcc
end
