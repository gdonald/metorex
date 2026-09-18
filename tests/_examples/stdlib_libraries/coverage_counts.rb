# Coverage counts how often each line of a file loaded while it is measuring
# runs. A line carrying nothing to run is left out of the count.
require "coverage"

p Coverage.supported? :lines
p Coverage.running?

Coverage.start
p Coverage.running?

load File.expand_path("coverage_counted_class.rb", __dir__)

p Coverage.result.values.first
p Coverage.running?

# Naming the modes outright reports each one under its own name. The methods
# mode names every method the file defines, called or not.
Coverage.start :all
load File.expand_path("coverage_counted_class.rb", __dir__)
measured = Coverage.result.values.first
p measured[:lines]
p measured[:branches]
p measured[:methods].map { |where, times| [where[1], where[2], where[4], times] }.sort
