# A subclass of String that writes its own `initialize` decides what its
# characters are, so `new` hands it arguments that need not be Strings.
class QualifiedName < String
  attr_reader :namespace

  def initialize(name, namespace = "")
    super(name.to_s)
    @namespace = namespace
  end

  def full_name
    self.class.new("#{namespace}#{self}")
  end
end

name = QualifiedName.new :Crate, "Shipping::"
p name
p name.namespace
p name.full_name
p name.full_name.class
p QualifiedName.new(:Crate) == "Crate"
p QualifiedName.new(42).length
