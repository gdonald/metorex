# A constant name may hold characters outside ASCII, and may open with an
# uppercase letter outside ASCII. Code eval'd in a block reads the block's
# locals, a module held in one included.

mod = Module.new
mod.const_set("BBἍBB", 1)
p(eval("mod::BBἍBB"))
p(mod::BBἍBB)

mod.const_set("ἍBB", 2)
p(eval("mod::ἍBB"))

def inside_a_block
  yield
end

inside_a_block do
  held = Module.new
  p(eval("held").class)
  held.const_set("CCἍCC", 3)
  p(eval("held::CCἍCC"))
end
