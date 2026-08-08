module Archive
  ONE = 1
  TWO = 2
end

Archive.deprecate_constant "ONE", :TWO
Archive.private_constant :ONE

puts Archive::TWO
puts Archive.const_get(:ONE)

begin
  Archive::ONE
rescue NameError
  puts "private"
end
