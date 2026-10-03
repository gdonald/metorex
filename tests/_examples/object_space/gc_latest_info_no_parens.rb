# GC.latest_gc_info reports the collector's last run: every reading in a new
# Hash, every reading added to a Hash given, or one reading by name. A run
# metorex makes is one the program asked for.
p GC.latest_gc_info.keys
GC.start
p GC.latest_gc_info
held = { kept: true }
p GC.latest_gc_info(held).equal? held
p held.size
p [GC.latest_gc_info(:gc_by), GC.latest_gc_info(:state)]
begin
  GC.latest_gc_info :unknown
rescue ArgumentError => error
  puts error.message
end
begin
  GC.latest_gc_info "state"
rescue TypeError => error
  puts error.message
end
