# A value that cannot stand in for an Integer is refused with a TypeError
# naming it: nil, true and false by name, and anything else by its class.
[nil, true, false, Object.new, "12"].each do |value|
  begin
    Integer.sqrt(value)
  rescue TypeError => error
    puts error.message
  end
end
