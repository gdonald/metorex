# Methods that hand a call on to an object an accessor names: an instance
# variable, a method, or an expression written as a String.
module Forwardable
  VERSION = "1.4.0"
  FORWARDABLE_VERSION = VERSION

  @debug = nil

  class << self
    attr_accessor :debug
  end

  # Each key names a method, or an Array of them, handed on to the accessor
  # its value names.
  def instance_delegate(hash)
    hash.each do |methods, accessor|
      if methods.respond_to?(:each)
        methods.each { |method| def_instance_delegator(accessor, method) }
      else
        def_instance_delegator(accessor, methods)
      end
    end
  end

  # `__send__` and `__id__` are left to the object itself.
  def def_instance_delegators(accessor, *methods)
    methods.each do |method|
      next if ["__send__", "__id__"].include?(method.to_s)
      def_instance_delegator(accessor, method)
    end
  end

  def def_instance_delegator(accessor, method, ali = method)
    source = Forwardable._delegator_method(self, accessor, method, ali)
    owner = Module === self ? self : singleton_class
    defined = owner.module_eval(source)
    owner.__send__(:ruby2_keywords, ali)
    defined
  end

  alias delegate instance_delegate
  alias def_delegators def_instance_delegators
  alias def_delegator def_instance_delegator

  # The source of a method named `ali` that hands its arguments and block to
  # `method` of what `accessor` answers. A method the accessor's object keeps
  # private is still reached, with a warning naming the call that reached it.
  def self._delegator_method(obj, accessor, method, ali)
    accessor = accessor.to_s unless Symbol === accessor
    named_method =
      if Module === obj
        obj.method_defined?(accessor) || obj.private_method_defined?(accessor)
      else
        obj.respond_to?(accessor, true)
      end
    accessor = "#{accessor}()" if named_method

    method_call = ".__send__(:#{method}, *args, &block)"
    before = ""
    if _valid_method?(method)
      location = caller_locations(2, 1).first
      owner = Module === obj ? obj : obj.class
      message = "#{owner}##{ali} at #{location.path}:#{location.lineno} forwarding to private method "
      before = "_ ="
      method_call = "\n" + <<~CALL
        unless defined? _.#{method}
          ::Kernel.warn #{message.dump}"\#{_.class}"'##{method}', uplevel: 1
          _#{method_call}
        else
          _.#{method}(*args, &block)
        end
      CALL
    end

    <<~METHOD
      def #{ali}(*args, &block)
        #{before}
        begin
          #{accessor}
        end#{method_call}
      end
    METHOD
  end

  # Whether `method` can be written as a call after a dot.
  def self._valid_method?(method)
    method.to_s.match?(/\A(?:[A-Za-z_][A-Za-z0-9_]*[?!=]?|\[\]=?|[-+]@|[-+*\/%<>=!~^&|]+|\*\*)\z/)
  end
  private_class_method :_valid_method?
end

# The same methods for one object, which hand its own calls on rather than
# those of instances of a class.
module SingleForwardable
  def single_delegate(hash)
    hash.each do |methods, accessor|
      if methods.respond_to?(:each)
        methods.each { |method| def_single_delegator(accessor, method) }
      else
        def_single_delegator(accessor, methods)
      end
    end
  end

  def def_single_delegators(accessor, *methods)
    methods.each do |method|
      next if ["__send__", "__id__"].include?(method.to_s)
      def_single_delegator(accessor, method)
    end
  end

  def def_single_delegator(accessor, method, ali = method)
    source = Forwardable._delegator_method(self, accessor, method, ali)
    defined = singleton_class.module_eval(source)
    singleton_class.__send__(:ruby2_keywords, ali)
    defined
  end

  alias delegate single_delegate
  alias def_delegators def_single_delegators
  alias def_delegator def_single_delegator
end
