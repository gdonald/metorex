# What `defined?` reports for each form, as a frozen string. A method call
# evaluates its receiver to ask about the method, and `&&`, `||`, and a
# string's interpolation are reported on without being run.
# Written with as few parentheses as Ruby allows.

p [defined?(self), defined?(nil), defined?(true), defined?(false)]
p defined?(self).frozen?
p [defined?([Object, Array]), defined?([Object, Missing])]

calls = []
recorder = Object.new
recorder.define_singleton_method(:note) { calls << :note; 5 }
p defined?(recorder.note / 2)
p defined?(recorder.note && true)
p defined?("value #{recorder.note}")
p calls

count = 42
p [defined?(count == 2), defined?(count !~ 2), defined?(!count), defined?(!@unset)]
p [defined?(count = 3), defined?(count += 1), defined?(@held ||= 1)]

$assigned_nil = nil
p [defined?($assigned_nil), defined?($never_assigned), defined?($~)]
"text" =~ /(x)/
p [defined?($&), defined?($1), defined?($2)]

class Parent
  def greet
    :parent
  end
end

class Child < Parent
  def greet
    [1].map { defined?(super) }
  end

  def farewell
    defined?(super)
  end
end
p [Child.new.greet, Child.new.farewell]

class Guarded
  def check(other)
    defined?(other.hidden)
  end

  protected

  def hidden; end
end
p [Guarded.new.check(Guarded.new), defined?(Guarded.new.hidden)]

p [defined?(if count then 1 end), defined?(__ENCODING__), defined?(1 / 0)]
