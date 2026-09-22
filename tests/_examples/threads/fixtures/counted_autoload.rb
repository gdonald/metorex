# Loaded once per name the autoload was registered under. The module is only
# finished after the pause, so a thread that reads it before then would see a
# module without its method.
which = $counted_loads.increment_and_get
eval <<-SOURCE
  module Counted#{which}
    sleep 0.02
    def self.ready
      :ready
    end
  end
SOURCE
