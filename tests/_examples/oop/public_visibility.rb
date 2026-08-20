class Ancestor
  private

  def redefined_later
    :before
  end
end

class Descendant < Ancestor
  public :redefined_later
end

class Ancestor
  def redefined_later
    :after
  end
end

puts Descendant.new.redefined_later.inspect
puts Descendant.public_instance_methods(false).inspect

class SameVisibility < Ancestor
  public :redefined_later
end
puts SameVisibility.public_instance_methods(false).inspect

class Toggling < Ancestor
  private :redefined_later
  public :redefined_later
end
puts Toggling.public_instance_methods(false).inspect
puts Toggling.private_instance_methods(false).inspect
