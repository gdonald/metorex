module Legacy
  OLD = :old
  KEPT = :kept
  HIDDEN = :hidden
end

Legacy.private_constant :HIDDEN
puts Legacy.deprecate_constant(:OLD).equal?(Legacy)

puts Warning[:deprecated]
puts Legacy::OLD.inspect
puts Legacy::KEPT.inspect
puts Legacy.const_get(:OLD).inspect

Warning[:deprecated] = true
puts Warning[:deprecated]
puts Legacy::OLD.inspect
Warning[:deprecated] = false

begin
  Legacy.deprecate_constant :MISSING
rescue NameError => e
  puts e.class
end

begin
  Legacy::HIDDEN
rescue NameError
  puts "private"
end

Legacy.public_constant :HIDDEN
puts Legacy::HIDDEN.inspect
