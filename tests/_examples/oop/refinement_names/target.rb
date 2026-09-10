# A refinement is a module of its own kind, and it names the class it refines.
Held = Class.new

refinement = nil
Module.new do
  refine Held do
    refinement = self
  end
end
p refinement.target
p refinement.refined_class
p Refinement.superclass

# A collection keeps an instance variable of its own, which a new collection
# built from it does not carry.
items = [1, 2, 3]
items.instance_variable_set "@label", "counted"
p items.instance_variable_get("@label")
p items.reject { false }.instance_variable_get("@label")

# A symbol's characters are written in ASCII when that is all they are.
p :abc.to_s.encoding
p "ä".to_sym.to_s.encoding

# A number names the character it stands for, in the encoding it is given.
p 65.chr.encoding
p 200.chr.encoding
p 0x3042.chr("UTF-8")
