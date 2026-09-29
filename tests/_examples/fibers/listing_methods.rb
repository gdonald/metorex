# Fiber#raise is a public method of Fiber itself.
p(Fiber.public_instance_methods(false).include?(:raise))
p(Fiber.instance_method(:raise).owner)
