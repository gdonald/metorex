# `callcc` and the continuations it hands its block, which return from the
# block with the values they are called with.
warn "#{__FILE__}: warning: callcc is obsolete; use Fiber instead" unless $VERBOSE.nil?

class Continuation
  class << self
    undef_method :new
  end

  # Return from the `callcc` block this continuation came from, which answers
  # nothing for no values, the value for one, and an Array for several. Once
  # the block has returned, the statement `callcc` was written in runs again
  # with `callcc` answering that, while the code holding the statement still
  # runs.
  def call(*values)
    answer = case values.size
             when 0 then nil
             when 1 then values.first
             else values
             end
    throw @tag, answer if @running
    unless @site && __continuation_resume__(@site[0], @site[1], answer)
      raise NotImplementedError,
            "a continuation cannot be resumed once the code its callcc was written in has returned"
    end
  end

  alias [] call

  def __start__(tag, site)
    @tag = tag
    @site = site
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
    site = __continuation_site__()
    if site && (resumed = __continuation_resumed__(site[0], site[1]))
      return resumed.first
    end
    tag = Object.new
    continuation = Continuation.allocate.__send__(:__start__, tag, site)
    begin
      catch(tag) { yield continuation }
    ensure
      continuation.__send__(:__finish__)
    end
  end
  private :callcc
end
