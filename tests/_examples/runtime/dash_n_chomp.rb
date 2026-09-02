$_ = "abc\n"
chomp
puts $_

$_ = "abc\r"
chomp
puts $_

$_ = "abc\r\n"
chomp
puts $_

$_ = "abc\n\n"
chomp
puts $_.inspect

$_ = "abcde"
$/ = "cde"
chomp
puts $_

$/ = "\n"
$_ = "abc\n"
chop
puts $_.inspect

$_ = "abc"
Kernel.chomp
puts $_

puts Kernel.private_method_defined? :chomp
puts Kernel.private_method_defined? :chop
