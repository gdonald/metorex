class GrandParent
  def initialize
    @level = "GrandParent"
  end

  def get_level
    @level
  end
end

class Parent < GrandParent
  def initialize
    super()
    @parent_level = "Parent"
  end

  def get_parent_level
    @parent_level
  end
end

class Child < Parent
  def initialize
    super()
    @child_level = "Child"
  end

  def get_child_level
    @child_level
  end
end

child = Child.new
puts child.get_level
puts child.get_parent_level
puts child.get_child_level
