module MyModule
  def self.class_method
    "from module"
  end
end

def MyModule.another_method
  "also from module"
end

puts MyModule.class_method
puts MyModule.another_method
