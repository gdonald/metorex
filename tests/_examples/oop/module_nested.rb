module Outer
  module Inner
    def self.greet
      "hello from Inner"
    end
  end

  class Widget
    def name
      "widget"
    end
  end
end

puts Outer::Inner.greet
puts Outer::Widget.new.name
