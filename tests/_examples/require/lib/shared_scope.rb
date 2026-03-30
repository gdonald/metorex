# Defines a variable, a function, and a class for scope sharing test
shared_var = "shared value"

def shared_function
  "shared function result"
end

class SharedClass
  def initialize
    @name = "SharedClass instance"
  end

  def name
    @name
  end
end
