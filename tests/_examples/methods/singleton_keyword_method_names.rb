class Widget
end

widget = Widget.new

def widget.class
  Integer
end

def widget.begin
  "opened"
end

def widget.nil
  "not really"
end

def widget.[](index)
  index * 2
end

def widget.<<(other)
  "shoveled #{other}"
end

puts widget.class
puts widget.begin
puts widget.nil
puts widget[21]
puts widget << "log"
puts widget.is_a?(Widget)

class Holder
  def initialize
    @target = Widget.new

    def @target.class
      Float
    end
  end

  def target_class
    @target.class
  end

  def target_is_a_widget
    @target.is_a?(Widget)
  end
end

holder = Holder.new
puts holder.target_class
puts holder.target_is_a_widget
