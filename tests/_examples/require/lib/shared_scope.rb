# Defines a constant, a method, and a class for the scope sharing test. A
# local variable would not be shared, since each file keeps its own.
SHARED_VALUE = "shared value"

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
