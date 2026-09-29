# encoding: binary
# Keywords spread from a Hash reach a method as the keywords they are, in a
# file whose text is bytes.
def gather(**keywords)
  keywords
end
options = { width: 3, height: 4 }
p(gather(**options))
p(gather(**options).keys)
