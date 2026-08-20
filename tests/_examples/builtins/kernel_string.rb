puts String("already").inspect
puts String(nil).inspect
puts String(1.12).inspect
puts String(true).inspect
puts String(false).inspect
puts String(42).inspect
puts String(Object).inspect
puts String :symbol

class Tag
  def to_s
    "tag"
  end
end

puts String(Tag.new)

class Silent
  undef_method :to_s
end

begin
  String(Silent.new)
rescue TypeError => e
  puts e.message
end

class Wrong
  def to_s
    123
  end
end

begin
  String(Wrong.new)
rescue TypeError => e
  puts e.message
end

original = "same"
puts String(original).equal?(original)

class Name < String
end

label = Name.new("metorex")
puts label.to_s
puts label.length
puts label.upcase
puts label == "metorex"
puts String(label).equal?(label)
puts Kernel.String(7)
puts Kernel.private_instance_methods.include?(:String)
