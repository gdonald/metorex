# `super` dispatched through an aliased method — the super call walks the
# ancestor chain from the method's original *defining* class, not the
# receiver's class. This makes `alias_method` + re-alias cycles work:

module Parent
  def talk(x)
    x
  end
end

module Child
  include Parent

  def talk(x)
    super(x)
  end
end

class Target
  include Child

  alias_method :alias_talk, :talk
  alias_method :talk, :alias_talk
end

puts Target.new.talk(42)
