# A class that has one instance, reached through `instance` and built no
# other way.
module Singleton
  def self.included(target)
    target.instance_variable_set(:@singleton__instance__, nil)
    class << target
      def instance
        held = instance_variable_get(:@singleton__instance__)
        return held unless held.nil?
        made = new_singleton_instance
        instance_variable_set(:@singleton__instance__, made)
        made
      end

      def new_singleton_instance
        made = allocate
        begin
          made.send(:initialize)
        rescue NoMethodError
        end
        made
      end
      private :new_singleton_instance

      def new(*args)
        raise NoMethodError, "private method 'new' called for class #{self}"
      end

      def allocate_singleton_copy
        raise TypeError, "can't copy singleton class"
      end
    end
    # A singleton is reached through `instance` alone, so the two ways of
    # building one are not among the class's own methods.
    target.private_class_method :allocate
  end

  def clone
    raise TypeError, "can't clone instance of singleton #{self.class}"
  end

  def dup
    raise TypeError, "can't dup instance of singleton #{self.class}"
  end

  def _dump(depth = -1)
    ""
  end
end
