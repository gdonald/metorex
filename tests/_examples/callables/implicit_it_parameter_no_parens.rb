# A block that names no parameters and mentions a bare `it` takes the first
# argument under that name. The parameter has no name of its own, so a proc
# reports it as optional and a lambda as required.
p proc { it }.call("a")
p -> { it }.call("b")
p ["c", "d"].map { it }
p proc { it }.parameters
p -> { it }.parameters
p -> { it }.arity
p proc { it }.call

# A name written as a method call is that call, not the parameter.
def it(described)
  described.upcase
end
p ["e"].map { it("f") }

# `lambda` is a method rather than syntax, so a program may name a local
# after it.
lambda = ->(first, second) { [first, second] }
p(lambda.call(1, 2))
p(lambda.parameters)
