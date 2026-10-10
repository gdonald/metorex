# Strings built by appending, interpolating and joining.
lines = []
50_000.times do |step|
  line = +"row "
  line << step.to_s << ": " << "value #{step * 3}"
  lines << line
end
puts lines.join("\n").bytesize
