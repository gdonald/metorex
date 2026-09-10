# The environment is looked up by text, so anything that reads as a String
# names a variable or a value.
class Named
  def initialize(text)
    @text = text
  end

  def to_str
    @text
  end
end

ENV["metorex_example"] = "held"
p(ENV.has_key?(Named.new("metorex_example")))
p(ENV.include?(Named.new("metorex_example")))
p(ENV.member?(Named.new("metorex_example")))
p(ENV.key?(Named.new("metorex_example")))
p(ENV.has_value?(Named.new("held")))
p(ENV.value?(Named.new("held")))
p(ENV.key(Named.new("held")))
p(ENV.assoc(Named.new("metorex_example")))
p(ENV.rassoc(Named.new("held")))

# A value that names nothing matches nothing, while a name that names nothing
# is refused.
p(ENV.has_value?(Object.new))
p(ENV.rassoc(Object.new))
begin
  ENV.has_key?(Object.new)
rescue TypeError => problem
  p(problem.message)
end

ENV.delete(Named.new("metorex_example"))
p(ENV["metorex_example"])
