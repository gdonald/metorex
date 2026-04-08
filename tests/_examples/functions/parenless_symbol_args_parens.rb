class Registry
  def self.register(event, handler)
    puts event.to_s
    puts handler
  end
end

Registry.register(:start, "handler_one")
Registry.register(:finish, "handler_two")
