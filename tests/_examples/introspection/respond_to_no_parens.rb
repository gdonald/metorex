class Vault
  def open_door
  end

  def hidden
  end
  private :hidden

  def guarded
  end
  protected :guarded
end

vault = Vault.new
puts vault.respond_to? :open_door
puts vault.respond_to? :hidden
puts vault.respond_to?(:hidden, true)
puts vault.respond_to? :guarded
puts vault.respond_to?(:guarded, true)
puts vault.respond_to? :nothing_here
puts vault.respond_to? "open_door"

class Sealed
  class << self
    private :new
  end
end

puts Sealed.respond_to? :new
puts Sealed.respond_to?(:new, false)
puts Sealed.respond_to?(:new, true)

begin
  Sealed.new
rescue NoMethodError => error
  puts "#{error.class}: #{error.message}"
end

puts Vault.respond_to? :new
puts Vault.respond_to? :instance_methods

class Named
  def to_str
    "open_door"
  end
end
puts vault.respond_to? Named.new

begin
  vault.respond_to? 42
rescue TypeError => error
  puts "#{error.class}: #{error.message}"
end
