# A call may spread a hash into the keywords and name more of them beside it.
# The one written later wins.
def settings(name, colour: nil, size: nil)
  [name, colour, size]
end

held = { colour: "red", size: 2 }
p settings("box", **held)
p settings("box", **held, size: 9)
p settings("box", size: 9, **held)

# A keyword the call leaves out is named in the message.
def needs(salt:, pepper:)
  [salt, pepper]
end

begin
  needs(salt: 1)
rescue ArgumentError => problem
  puts problem.message
end

begin
  needs
rescue ArgumentError => problem
  puts problem.message
end
