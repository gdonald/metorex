# An `alias` written where a `def` would land on the receiver alone lands
# there too, so the second name belongs to that object and to nothing else.
class Reading
  def value
    5
  end
end

held = Reading.new
held.instance_eval do
  alias second value
end

p(held.second)
p(held.singleton_methods)
p(Reading.new.respond_to?(:second))

# A value the program cannot hold one copy of has no singleton class to
# write the second name on.
begin
  :name.instance_eval do
    alias spelled to_s
  end
rescue TypeError => error
  puts(error.message)
end

# A name written at the top level belongs to Object, and so does the second
# name an alias gives it.
def counted
  7
end
alias tallied counted
p(method(:tallied).owner)
p(tallied)
