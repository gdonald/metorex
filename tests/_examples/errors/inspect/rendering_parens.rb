# `Exception#inspect` reads `#<ClassName: message>`, using whatever `to_s`
# answers, and the class name alone when that is empty.

puts(Exception.new.inspect)
puts(Exception.new("boom").inspect)
puts(RuntimeError.new("boom").inspect)
puts(RuntimeError.new("").inspect)

class Described < StandardError
  def to_s
    "this is from to_s"
  end
end

puts(Described.new.inspect)

class Silent < StandardError
  def to_s
    ""
  end
end

puts(Silent.new("ignored").inspect)

class Plain < StandardError
end

puts(Plain.new.inspect)

# An anonymous class shows its generated label.
anonymous = Class.new(RuntimeError)
puts(anonymous.new("message").inspect.start_with?("#<#<Class:").to_s)
