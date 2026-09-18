# `#@name`, `#@@name`, and `#$name` interpolate without braces, and a `#`
# followed by anything else stands as it reads.
$global = "gee"

class Holder
  @@shared = "shared"

  def initialize
    @held = "held"
    @held_again = "again"
  end

  def written
    ["#@held", "#$global", "#@held_again", "#@@shared", "#@held[", "#@held#@held",
     "#@", "#@ ", "#@@", "#$%"]
  end
end

puts Holder.new.written.inspect

# `\cX`, `\C-X`, and `\M-X` name a control or meta character.
puts "\cx".bytes.inspect
puts "\C-x".bytes.inspect
puts "\M-x".bytes.inspect
puts "\M-\C-x".bytes.inspect
puts ?\C-z.bytes.inspect

# Two pieces that each hold something outside ASCII have to be written in the
# same encoding.
begin
  wide = "あ"
  raw = "\xff".dup.force_encoding "binary"
  puts "#{wide} #{raw}"
rescue Encoding::CompatibilityError => refused
  puts refused.class
end

# At the top level an instance variable belongs to main.
@top = "main's own"
puts "#@top"
puts @never_written.inspect
