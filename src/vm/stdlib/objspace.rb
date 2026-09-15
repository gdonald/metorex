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

  # What every object of a kind holds between them. Metorex counts what the
  # program has built rather than walking a heap, so this grows as the
  # program makes objects and never falls.
  # Everything one object refers to directly: the class it belongs to, what
  # its instance variables hold, and the elements or pairs it stores. The
  # singletons and the immediate values refer to nothing, so they answer nil.
  def reachable_objects_from(object)
    return nil if object.nil? || object.equal?(true) || object.equal?(false)
    return nil if object.is_a?(Integer) || object.is_a?(Symbol) || object.is_a?(Float)
    found = [object.class]
    object.instance_variables.each do |name|
      found.push object.instance_variable_get(name)
    end
    if object.is_a? Array
      found.concat object
    elsif object.is_a? Hash
      object.each { |key, value| found.push(key).push(value) }
    end
    found
  end

  def memsize_of_all(kind = nil)
    built = ObjectSpace.__allocated__(kind)
    # A class is an object too, and the ones the interpreter starts with are
    # there before the program builds anything.
    built = built + 1 if kind.nil? || kind == Class || kind == Module
    built * 40
  end
end
