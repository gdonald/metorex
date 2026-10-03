# A C extension keeping objects alive through the collector's registration
# functions, switching collection off and on, running it, and reading what
# its last run was.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_collection.c", "c_collection", directory)
collection = CCollection.new

collection.keep "kept text"
GC.start
p collection.held
p collection.release
p collection.switched
p collection.runs
p collection.latest :gc_by
report { collection.latest :unknown }
report { collection.latest "gc_by" }
p collection.sizes 42

FileUtils.rm_rf directory
