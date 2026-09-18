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

module ObjectSpace
  module_function

  # What one object is, written as JSON. Metorex has no heap to walk, so the
  # dump names what the object itself can answer: what kind it is, the class
  # it belongs to, and what it holds.
  def dump(object, output: :string)
    text = ObjectSpace.__dump_text__ object
    case output
    when :string then text
    when :stdout
      STDOUT.puts text
      nil
    when :file, nil then ObjectSpace.__dump_to_file__ text
    else
      unless output.respond_to?(:write)
        raise ArgumentError, "wrong output option: #{output.inspect}"
      end
      output.write text
      output
    end
  end

  # Every object the program has built, one dump to a line. Metorex frees an
  # object when the last reference to it goes, so what is reachable from the
  # program is what a walk can reach.
  def dump_all(output: :file)
    lines = ObjectSpace.__live_objects__.map { |held| ObjectSpace.__dump_text__ held }
    text = lines.join("\n") + "\n"
    case output
    when :string then text
    when :stdout
      STDOUT.puts text
      nil
    when :file, nil then ObjectSpace.__dump_to_file__ text
    else
      unless output.respond_to?(:write)
        raise ArgumentError, "wrong output option: #{output.inspect}"
      end
      output.write text
      output
    end
  end

  def __dump_to_file__(text)
    require "tmpdir"
    path = File.join Dir.tmpdir, "objspace_dump_#{Process.pid}_#{rand(1 << 32).to_s(16)}"
    file = File.open path, "w+"
    file.write text
    file.flush
    file.rewind
    file
  end

  def __dump_text__(object)
    fields = ["\"address\":\"#{ObjectSpace.__address_of__(object)}\""]
    fields.push "\"type\":\"#{ObjectSpace.__dump_type__(object)}\""
    fields.push "\"class\":\"#{ObjectSpace.__address_of__(object.class)}\"" unless object.nil?
    fields.concat ObjectSpace.__dump_body__(object)
    fields.push "\"memsize\":#{ObjectSpace.memsize_of(object)}"
    fields.push "\"frozen\":true" if object.frozen?
    "{#{fields.join(", ")}}"
  end

  # The fields that belong to the kind of object this is.
  def __dump_body__(object)
    case object
    when String
      [
        "\"bytesize\":#{object.bytesize}",
        "\"value\":#{ObjectSpace.__json_string__(object)}",
        "\"encoding\":\"#{object.encoding.name}\"",
      ]
    when Symbol then ["\"value\":#{ObjectSpace.__json_string__(object.to_s)}"]
    when Array then ["\"length\":#{object.length}"]
    when Hash then ["\"size\":#{object.size}"]
    when Float then ["\"value\":\"#{object}\""]
    when Integer then ["\"value\":\"#{object}\""]
    when Class, Module
      named = object.name
      held = ["\"name\":#{named.nil? ? "null" : ObjectSpace.__json_string__(named)}"]
      if object.is_a?(Class) && object.superclass
        held.push "\"superclass\":\"#{ObjectSpace.__address_of__(object.superclass)}\""
      end
      held
    else ["\"ivars\":#{object.instance_variables.size}"]
    end
  end

  # The kind name MRI writes, which names the same kinds metorex has.
  def __dump_type__(object)
    case object
    when nil, true, false then "NIL_OR_BOOL"
    when String then "STRING"
    when Symbol then "SYMBOL"
    when Array then "ARRAY"
    when Hash then "HASH"
    when Float then "FLOAT"
    when Integer then "BIGNUM"
    when Class then "CLASS"
    when Module then "MODULE"
    when Proc then "DATA"
    else "OBJECT"
    end
  end

  # Where an object sits, written the way a dump names it. Metorex has no
  # heap to point into, so the object's own id stands for its place, which is
  # as unique and as stable as an address is.
  def __address_of__(object)
    format "0x%016x", object.object_id
  end

  # A string written the way JSON writes one, which is what the dump is.
  def __json_string__(text)
    require "json"
    JSON.generate text
  end
end
