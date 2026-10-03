# Requiring mkmf defines MakeMakefile and includes it at the top level.
require "mkmf"

p MakeMakefile
p Object.include? MakeMakefile
