# `lambda` is a method rather than a keyword, so a parameter may take its
# name and the body reads it as that parameter.

def pick value, lambda
  lambda ? [:lambda, value] : [:proc, value]
end

p pick(1, true)
p pick(2, false)
