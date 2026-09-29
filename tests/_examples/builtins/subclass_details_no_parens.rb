# Details Marshal reads from the values it writes.
p Encoding.find("sjis")
p Encoding::CP932

class Text < String; end
p Text.new.encoding

pattern = Regexp.new("")
pattern.instance_variable_set :@note, :kept
p pattern.instance_variable_get(:@note)

class Pattern < Regexp; end
p Pattern.new("a").encoding

class Counts < Hash
  def initialize(label)
    @label = label
  end
end
p Counts.new(7).default
