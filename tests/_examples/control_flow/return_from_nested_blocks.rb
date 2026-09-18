# A `return` written inside a block unwinds to the method the block was
# written in, however many blocks and calls stand between them.
class Runner
  def yielding_method
    yield
    :after_yield
  end

  def nested
    yielding_method do
      yielding_method do
        return :from_the_inner_block
      end
      :after_inner
    end
    :after_outer
  end

  def saved
    outer do
      inner do
        keep do
          return :from_the_saved_block
        end
      end
    end
    :bottom
  end

  def keep(&block)
    @kept = block
  end

  def outer
    yield
    @kept.call
  end

  def inner
    yield
  end
end

puts Runner.new.nested.inspect
puts Runner.new.saved.inspect

# A `return` written in a class body is a syntax error, and one written in a
# block inside a class body has no method to return from.
begin
  eval "class ReturnInBody; return; end"
rescue SyntaxError => refused
  puts "SyntaxError"
end

begin
  eval "class ReturnInBlockInBody; 1.times { return }; end"
rescue LocalJumpError => refused
  puts refused.message
end
