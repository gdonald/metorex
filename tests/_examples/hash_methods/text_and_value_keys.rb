# A String key that reads as a number, a boolean or nil is a key of its own,
# apart from the value it reads as, since the two are not `eql?`.
mixed = {"646" => 1, 646 => 2, "true" => 3, true => 4, "nil" => 5, nil => 6, "1.5" => 7, 1.5 => 8}
p(mixed)
p(mixed.keys)
p([mixed["646"], mixed[646], mixed.key?("646"), mixed.key?("647")])
mixed.delete("646")
p(mixed.first)

built = {}
built["10"] = :text
built[10] = :number
p(built)
p({"1" => 2}.merge({1 => 3}))
p([["1", 1], [1, 2]].to_h)
p({"1" => 1}.transform_keys(&:to_i))
