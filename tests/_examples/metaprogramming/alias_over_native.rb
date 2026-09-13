# The `alias` keyword points a second name at a method the interpreter answers
# natively, so the original stays reachable after a redefinition replaces it.
class Integer
  alias old_spaceship <=>

  def <=>(other)
    raise "replaced"
  end
end

p 5.old_spaceship(6)

begin
  1 <=> 2
rescue RuntimeError => trouble
  p trouble.message
end

class Integer
  alias <=> old_spaceship
end

p(1 <=> 2)
p [3, 1, 2].sort

class Numbering
  alias_method :counted, :to_s
end
p Numbering.new.counted.start_with?("#<Numbering")
