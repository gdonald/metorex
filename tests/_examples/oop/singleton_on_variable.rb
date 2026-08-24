class Box
  def initialize
    @item = Object.new

    def @item.special
      "the one"
    end
  end

  def item
    @item
  end
end

puts Box.new.item.special

$stream = Object.new

def $stream.write(text)
  "wrote #{text}"
end

puts $stream.write "data"

plain = Object.new
puts plain.respond_to?(:special)
