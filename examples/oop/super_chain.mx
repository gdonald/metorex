class GrandParent
  def initialize
    @level = "GrandParent"
  end

  attr_reader :level
end

class Parent < GrandParent
  def initialize
    super()
    @parent_level = "Parent"
  end

  attr_reader :parent_level
end

class Child < Parent
  def initialize
    super()
    @child_level = "Child"
  end

  attr_reader :child_level
end

child = Child.new
puts child.level
puts child.parent_level
puts child.child_level
