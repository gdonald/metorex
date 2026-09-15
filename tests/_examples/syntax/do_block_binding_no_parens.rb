# A `do ... end` block belongs to the outermost command, where a `{ }` block
# belongs to the call it sits next to.
class Outer
  def takes(first, second)
    held = yield
    "outer got #{held}"
  end
end

class Inner
  def permitted?
    "permitted"
  end

  def takes_a_block(name)
    block_given? ? "inner took it" : "inner had none"
  end
end

outer = Outer.new
inner = Inner.new
p(outer.takes "a", inner.permitted? do
  "the block"
end)
p outer.takes("a", inner.takes_a_block("x") { "brace" }) { "the block" }
