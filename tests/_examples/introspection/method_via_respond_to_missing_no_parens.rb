class Ghost
  def respond_to_missing?(name, include_private = false)
    name == :haunt || (name == :whisper && include_private)
  end

  def method_missing(name, *args)
    "called #{name} with #{args.inspect}"
  end
end

ghost = Ghost.new
haunt = ghost.method(:haunt)
puts haunt.class.name
puts haunt.call
puts haunt.call 1, 2
whisper = ghost.method :whisper
puts whisper.call("softly")

begin
  ghost.method :unknown
rescue NameError => error
  puts "#{error.class}: #{error.message}"
end

class OneArgumentMissing
  def respond_to_missing?(name, include_private = false)
    name == :only_name
  end

  def method_missing(name)
    name
  end
end

only_name = OneArgumentMissing.new.method :only_name
puts only_name.call.inspect

begin
  OneArgumentMissing.new.method(:only_name).call 1
rescue ArgumentError => error
  puts error.class
end

class Named
  def to_str
    "upcase"
  end
end

upcaser = "shout".method Named.new
puts upcaser.call

begin
  "shout".method nil
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end
