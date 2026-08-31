class Vault
  def open_door
    :opened
  end

  def hidden
    :hidden
  end
  private :hidden

  def guarded
    :guarded
  end
  protected :guarded

  def self.build
    :built
  end
end

vault = Vault.new
puts vault.public_method(:open_door).call.inspect
puts Vault.public_method(:build).call.inspect

[:hidden, :guarded].each do |name|
  begin
    vault.public_method(name)
  rescue NameError => error
    puts "#{error.class}: #{error.message}"
  end
end

class Ghost
  def respond_to_missing?(name, include_private = false)
    return true if name == :publicly_handled
    include_private && name == :privately_handled
  end

  def method_missing(name, *args)
    "called #{name}"
  end
end

ghost = Ghost.new
puts ghost.public_method(:publicly_handled).call
puts ghost.method(:privately_handled).call

begin
  ghost.public_method(:privately_handled)
rescue NameError => error
  puts error.class
end
