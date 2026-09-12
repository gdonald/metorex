# A Struct writes its own `initialize`: it is private, assigns the members it
# is given, and a struct class built from a subclass of Struct stands under
# that subclass.
Point = Struct.new :x, :y

p Point.private_instance_methods(false).include?(:initialize) ||
  Struct.private_instance_methods(false).include?(:initialize)

here = Point.new 1, 2
here.instance_eval { initialize 3, 4 }
p [here.x, here.y]

here.instance_eval { initialize 5 }
p [here.x, here.y]

class Tagged < Struct
  attr_reader :tag

  def initialize(*)
    @tag = :tagged
    super
  end
end

Labelled = Tagged.new :name
held = Labelled.new "held"
p [held.name, held.tag]
p Labelled.superclass == Tagged
