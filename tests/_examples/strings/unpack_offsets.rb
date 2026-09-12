# `unpack` reads from a byte offset when it is given one, and the offset must
# name a place inside the string.
held = "ZZABCD"
p held.unpack1 "x3C", offset: 2
p held.unpack "a4", offset: 2
p "ZZZZaG9nZWZ1Z2E=".unpack1 "m", offset: 4
p "؈".unpack "CC"
p "؈".unpack1 "C", offset: 1
p "a".unpack1 "C", offset: 1

begin
  "a".unpack1 "C", offset: -1
rescue ArgumentError => trouble
  p trouble.message
end

begin
  "a".unpack1 "C", offset: 2
rescue ArgumentError => trouble
  p trouble.message
end
