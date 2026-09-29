def kind event
  case event
  when :call, :return then "method"
  when :if, :end, :class then "keyword"
  else "other"
  end
end

puts kind :return
puts kind :end
puts kind :line

case :nil
in :nil then puts "pattern"
end
