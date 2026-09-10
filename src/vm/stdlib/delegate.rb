# Delegation. A Delegator stands in front of another object and passes on
# what it is asked, so a class can add to an object's behaviour without
# inheriting from its class.

class Delegator
  def initialize(object)
    __setobj__ object
  end

  # The object being stood in front of. A subclass says how it is held.
  def __getobj__
    raise NotImplementedError, "#{self.class} does not implement __getobj__"
  end

  def __setobj__(_object)
    raise NotImplementedError, "#{self.class} does not implement __setobj__"
  end

  def method_missing(name, *arguments, &block)
    target = __getobj__
    if target.respond_to? name
      target.__send__ name, *arguments, &block
    else
      super
    end
  end

  # A delegator answers to whatever the object behind it answers to. A
  # private or protected name is reported as one it does not forward, which
  # is a warning rather than an error so the caller can see why.
  def respond_to_missing?(name, include_private = false)
    target = __getobj__
    answered = target.respond_to? name, include_private
    if answered && include_private && !target.respond_to?(name, false)
      warn "delegator does not forward private method ##{name}"
      return false
    end
    answered
  end

  def methods(all = true)
    __getobj__.methods(all) | super
  end

  def public_methods(all = true)
    __getobj__.public_methods(all) | super
  end

  def protected_methods(all = true)
    __getobj__.protected_methods(all) | super
  end

  # Private methods are the delegator's own. Nothing private travels across
  # the boundary, so the object behind it contributes none.
  def private_methods(all = true)
    super
  end

  # `method` looks at the delegator first and then at the object behind it,
  # which only hands over what it would answer publicly.
  def method(name)
    begin
      super
    rescue NameError
      target = __getobj__
      unless target.respond_to? name
        raise NameError, "undefined method '#{name}' for #{self.inspect}"
      end
      target.method name
    end
  end

  # Comparison against the delegator itself is settled here, since the object
  # behind it has never heard of the thing standing in front of it.
  def ==(other)
    return true if other.equal? self
    __getobj__ == other
  end

  def !=(other)
    return false if other.equal? self
    __getobj__ != other
  end

  def !
    !__getobj__
  end

  def ~
    ~__getobj__
  end

  def <=>(other)
    __getobj__ <=> other
  end

  def ===(other)
    __getobj__ === other
  end

  def =~(other)
    __getobj__ =~ other
  end

  def eql?(other)
    return true if other.equal? self
    __getobj__.eql? other
  end

  def hash
    __getobj__.hash
  end

  def to_s
    __getobj__.to_s
  end

  def inspect
    __getobj__.inspect
  end

  # `tap` yields the delegator, since the point of it is to hold on to the
  # object the caller has in hand.
  def tap
    yield self
    self
  end

  # Freezing a delegator freezes the object behind it too, so a frozen one
  # refuses every way of writing through it.
  def freeze
    __getobj__.freeze
    super
  end

  def marshal_dump
    [:__v2__, instance_variables_to_dump, __getobj__]
  end

  def marshal_load(held)
    _version, variables, object = held
    variables.each { |name, value| instance_variable_set name, value }
    __setobj__ object
  end

  # The delegator's own state, leaving out the slot the object behind it
  # lives in, which is dumped alongside rather than inside it.
  def instance_variables_to_dump
    kept = {}
    instance_variables.each do |name|
      next if name == :@delegate_sd_obj
      kept[name] = instance_variable_get(name)
    end
    kept
  end

  private :instance_variables_to_dump

  # The names a delegating class carries in its own right, which is what
  # `DelegateClass` asks for when it builds one.
  def self.public_api
    public_instance_methods
  end
end

# A delegator that holds the object in an instance variable, which is the
# form most callers want.
class SimpleDelegator < Delegator
  def __getobj__
    unless defined? @delegate_sd_obj
      raise ArgumentError, "not delegated"
    end
    @delegate_sd_obj
  end

  def __setobj__(object)
    raise ArgumentError, "cannot delegate to self" if object.equal? self
    @delegate_sd_obj = object
  end
end

# `DelegateClass(Klass)` builds a class that delegates Klass's own methods,
# so the names it answers to are known before any object stands behind it.
def DelegateClass(superclass, &block)
  built = Class.new(Delegator)
  built.class_eval do
    define_method :__getobj__ do
      unless defined? @delegate_dc_obj
        raise ArgumentError, "not delegated"
      end
      @delegate_dc_obj
    end

    define_method :__setobj__ do |object|
      raise ArgumentError, "cannot delegate to self" if object.equal? self
      @delegate_dc_obj = object
    end
  end
  # Every method the delegated class carries, public and protected alike, is
  # written onto the built class so it answers to them before any object is
  # standing behind it.
  carried = superclass.instance_methods | superclass.protected_instance_methods
  # What every object already answers stays where it is. Forwarding `send`
  # or `frozen?` would send them to an object that is not there yet, so a
  # plain object is asked what it answers rather than reading a list.
  plain = Object.new
  left_alone = Delegator.instance_methods
  carried.each do |name|
    next if left_alone.include? name
    next if plain.respond_to? name, true
    built.class_eval do
      define_method name do |*arguments, &given|
        __getobj__.__send__ name, *arguments, &given
      end
    end
  end
  superclass.protected_instance_methods.each do |name|
    next if left_alone.include?(name) || plain.respond_to?(name, true)
    built.send :protected, name
  end
  built.class_eval(&block) if block
  built
end
