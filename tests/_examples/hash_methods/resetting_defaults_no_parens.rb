# `initialize` is the hook behind `Hash.new`, and calling it again sets a new
# default without touching the pairs already stored.
held = { "one" => 1 }
p held.default

held.send :initialize, 0
p [held.default, held["missing"], held.to_a]

held.send :initialize do |hash, key|
  key.to_s * 2
end
p [held.default, held["ab"], held.to_a]

held.send :initialize
p [held.default, held.default_proc]

made = Set.new [1, 2, 2, 3]
p [made.size, made.include?(2)]

begin
  Set.new 42
rescue ArgumentError => refused
  p refused.message
end
