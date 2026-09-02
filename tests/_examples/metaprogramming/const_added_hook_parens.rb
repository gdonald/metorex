class Tracker
  ADDED = []

  def self.const_added(name)
    ADDED << name
    puts(ADDED.inspect)
  end

  TEST = 1
  SECOND = 2

  autoload(:Autoload, "tracker/autoload.rb")

  class Child
  end
end

Tracker.const_set(:DIRECT, 4)
