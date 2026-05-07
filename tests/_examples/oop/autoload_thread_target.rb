module Outer
  class Foo
    block = $saved_lambda
    block.call
  end
end
