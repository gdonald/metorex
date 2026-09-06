# A symbol can name any operator method, which is how `send` reaches one.
p(6.send(:/, 2))
p(2.send(:**, 3))
p(6.send(:&, 3))
p(6.send(:|, 1))
p(2.send(:-@))
p(2.send(:+@))
p(-2.send(:-@))
p(2.5.send(:-@))
p(:/)
p(:**)
p(:-@)
p(:!)

# A colon with anything between it and the slash still opens a regular
# expression, in a ternary and in a hash alike.
p(true ? 1 : /re/.source)
p({ key: /ab/ }.keys)
