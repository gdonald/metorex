# Each Ractor keeps values of its own under Symbol keys, which only that
# Ractor reads and writes. `Ractor.store_if_absent` stores what its block
# answers the first time a key is asked for.

Warning[:experimental] = false
Ractor[:setting] = 1
p(Ractor[:setting], Ractor.current[:setting], Ractor["setting"])
Ractor.current[:other] = 2
p(Ractor[:other])
r = Ractor.new { [Ractor[:setting], (Ractor[:mine] = 3), Ractor[:mine]] }
p(r.value)
p(Ractor[:mine])
p(Ractor.store_if_absent(:made) { |key| [key, :first] })
p(Ractor.store_if_absent(:made) { |key| [key, :second] })
begin; r[:x]; rescue => e; p([e.class, e.message]); end
begin; r[:x] = 1; rescue => e; p([e.class, e.message]); end
begin; Ractor[1]; rescue => e; p([e.class, e.message]); end
begin; Ractor.store_if_absent(:y); rescue => e; p([e.class, e.message]); end
Ractor.store_if_absent("k") { 5 }
p(Ractor[:k])
p(Ractor.current.instance_variables, r.instance_variables, Ractor::Port.new.instance_variables)
