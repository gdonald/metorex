class Widget
  attr_accessor :parts

  def initialize
    @parts = ["a"]
  end
end

widget = Widget.new
def widget.special
  :the_one
end

copy = widget.clone
puts(copy.special)
puts(widget.dup.respond_to?(:special))

frozen = Widget.new.freeze
puts(frozen.clone.frozen?)
puts(frozen.clone(freeze: false).frozen?)
puts(Widget.new.clone(freeze: true).frozen?)
puts(Widget.new.clone(freeze: nil).frozen?)

begin
  widget.clone(freeze: 1)
rescue ArgumentError => error
  puts error.message
end

class Tracked
  def initialize_clone(other, **options)
    $recorded = options
  end
end

Tracked.new.clone(freeze: true)
puts($recorded[:freeze])

class OnlyOne
  def initialize_clone(other)
    :ignored
  end
end

begin
  OnlyOne.new.clone(freeze: true)
rescue ArgumentError => error
  puts error.message
end

base = Class.new do
  def label
    ["base"]
  end
end

object = base.new
object.define_singleton_method(:label) do
  ["singleton", *super()]
end
p(object.label)
p(object.clone.label)
