# A C extension making and reading Floats, Complex numbers and Rationals.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_numerics.c", "c_numerics", directory)
numerics = CNumerics.new

p numerics.half
p numerics.doubled 1.25
report { numerics.doubled 2 }
p [numerics.is_float(1.0), numerics.is_float(1), numerics.is_float(nil)]
p numerics.to_float "2.5"
p numerics.to_float 3
report { numerics.to_float "many" }

p numerics.complex 1, 2
p numerics.complex "3", "4"
p numerics.complex_real 5
p numerics.complex_new 10, 4
p numerics.complex_new_real 6
p numerics.complex_new 1.5, 2

p numerics.rational 1, 2
p numerics.rational "3", "4"
p numerics.rational_whole 5
p numerics.rational_new 10, 4
p numerics.rational_new_whole 6
report { numerics.rational_new 1, 0 }
p numerics.parts Rational(7, 2)

FileUtils.rm_rf directory
