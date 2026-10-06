# nil, true, false and a String answer to_f and to_r, yet a number is not
# read from them where Ruby wants one.
attempts = {
  "1.coerce(nil)" => -> { 1.coerce nil },
  "1.coerce(true)" => -> { 1.coerce true },
  "1.coerce(:name)" => -> { 1.coerce :name },
  "1.coerce(1..2)" => -> { 1.coerce 1..2 },
  "1.fdiv(nil)" => -> { 1.fdiv nil },
  "1.fdiv(\"2\")" => -> { 1.fdiv "2" },
  "[nil].pack(\"D\")" => -> { [nil].pack "D" },
  "[\"1.5\"].pack(\"e\")" => -> { ["1.5"].pack "e" },
  "[true].pack(\"f\")" => -> { [true].pack "f" },
  "sleep(\"2\")" => -> { sleep "2" },
  "Time.at(0) + nil" => -> { Time.at(0) + nil },
  "Time.at(0) - nil" => -> { Time.at(0) - nil },
}
attempts.each do |written, attempt|
  attempt.call
rescue TypeError => error
  puts "#{written}: #{error.message}"
end
p 1.coerce("2")
