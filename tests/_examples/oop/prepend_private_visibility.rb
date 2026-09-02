module Wrapper
  def wrapped
    super + 1
  end
end

class Counter
  prepend Wrapper

  def wrapped
    1
  end
  private :wrapped
end

counter = Counter.new
puts counter.wrapped

module QuietWrapper
  def hidden
    :wrapped
  end
  private :hidden
end

class Vault
  prepend QuietWrapper

  def hidden
    :own
  end
end

begin
  Vault.new.hidden
rescue NoMethodError => error
  puts error.message
end
