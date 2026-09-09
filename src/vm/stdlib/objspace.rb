# What the object space can say about the objects in it. Metorex frees an
# object when the last reference to it goes, so the sizes reported here are
# counted from what an object holds rather than measured in the heap.
module ObjectSpace
  module_function

  # The three singletons, the numbers, and the symbols live for the whole
  # run and are counted as holding nothing.
  def memsize_of(object)
    return 0 if object.nil? || object.equal?(true) || object.equal?(false)
    return 0 if object.is_a?(Integer) || object.is_a?(Symbol)
    counted = 40
    if object.respond_to?(:instance_variables)
      counted += object.instance_variables.size * 8
    end
    counted += object.size * 8 if object.is_a?(Array)
    counted += object.length if object.is_a?(String)
    counted
  end

  def memsize_of_all(kind = nil)
    0
  end
end
