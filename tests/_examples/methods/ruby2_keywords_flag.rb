class Forwarder
  ruby2_keywords def forwards(*args)
    args
  end
end

collected = Forwarder.new.forwards(1, 2)
puts collected.length
puts collected.last

begin
  Forwarder.class_eval { ruby2_keywords :missing }
rescue NameError => error
  puts error.message
end

class Unqualified
  def positional(first, second)
    first
  end
end

Unqualified.class_eval { ruby2_keywords :positional }
puts "warned and carried on"
