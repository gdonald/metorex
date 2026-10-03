# A C extension defining Struct and Data classes, making and reading their
# instances, and filling in instances that were only allocated.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_structs.c", "c_structs", directory)
structs = CStructs.new

named = structs.define "Pair"
p named
p Struct::Pair.members
anonymous = structs.define nil
p anonymous.name
report { structs.define "lower" }
report { structs.define_twice }

module Holder
end
under = structs.define_under Holder, "Inner"
p under
p structs.define_under(Holder, "Inner").equal? under
Holder.const_set :Plain, Class.new
report { structs.define_under Holder, "Plain" }

Shape = structs.define_data nil
shape = Shape
p shape.superclass
base = Class.new Data
p structs.define_data(base).superclass.equal? base
report { structs.define_data [] }
report { structs.define_data_twice }

pair = structs.make named, 1, 2
p pair
p structs.make_with(named, [3])
p structs.class_members named
p structs.members pair
p structs.size pair
p [structs.read(pair, :left), structs.read(pair, "right"), structs.read(pair, 1)]
p structs.write(pair, :left, 10)
p pair.left
report { structs.read pair, :missing }
report { structs.read pair, 2 }
report { structs.read pair, -3 }
p structs.member(pair, :right)
report { structs.member pair, :missing }

p structs.fill(pair, [7, 8])
p pair
report { structs.fill pair, [1, 2, 3] }
pair.freeze
report { structs.fill pair, [1] }

empty = shape.allocate
p structs.fill(empty, [4])
p [empty.width, empty.height, empty.frozen?]
report { structs.fill empty, [1, 2] }
report { structs.fill shape.allocate, [1, 2, 3] }

FileUtils.rm_rf directory
