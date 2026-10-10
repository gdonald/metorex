$LOAD_PATH.unshift File.join(__dir__, "autoload_fixture")

module Shipping
  autoload :Carrier, "shipping_carrier"

  module Ground
    class Truck < Carrier
      def name = "truck after #{super}"
    end
  end
end

p Shipping::Ground::Truck.new.name
p Shipping::Ground::Truck.superclass
