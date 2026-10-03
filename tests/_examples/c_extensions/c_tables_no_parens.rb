# A C extension keeping its own data in st_table hash tables keyed by
# numbers, by strings, by strings without case, and by a type of its own.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_tables.c", "c_tables", directory)
tables = CTables.new

p tables.numbers
p tables.strings
p tables.custom

FileUtils.rm_rf directory
