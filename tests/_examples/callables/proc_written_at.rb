# A block written in the main script names that script the way it was given
# on the command line, as `__FILE__` does.

written = proc { :inside }
p(written.source_location[0] == __FILE__)
p(written.inspect.include?("#{__FILE__}:4"))
