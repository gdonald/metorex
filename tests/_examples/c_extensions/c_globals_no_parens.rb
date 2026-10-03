# A C extension defining globals whose values live in C memory or come from
# C functions, and reading and writing Ruby's globals by name.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require build_extension("c_globals.c", "c_globals", directory)
globals = CGlobals.new
globals.define_all

p $c_stored
$c_stored = "written from Ruby"
p globals.stored_now
p $c_fixed
report { eval "$c_fixed = 1" }
$c_doubled = 21
p $c_doubled
p $c_defaults
p $c_nowhere
$c_nowhere = :ignored
p $c_nowhere
p $c_virtual
report { eval "$c_virtual = 1" }
p $c_counting
p $c_counting
$c_counting = 100
p $c_counting
traced = []
trace_var(:$c_doubled) { |value| traced << value }
$c_doubled = 4
untrace_var :$c_doubled
p [traced, $c_doubled]

p [globals.get("$c_doubled"), globals.get("c_doubled")]
p globals.set("$from_c", :set)
p $from_c
p globals.get "from_c"
p globals.names.include?(:$c_counting)
p globals.last_line "the last line"
p $_
p globals.separators
$; = ","
$, = "-"
$\ = "!"
p globals.separators
$; = nil
$, = nil
$\ = nil
p globals.separators.last.frozen?
p globals.streams.map(&:fileno)

FileUtils.rm_rf directory
