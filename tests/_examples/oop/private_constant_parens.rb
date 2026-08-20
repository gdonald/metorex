class Holder
  VISIBLE = :visible
  HIDDEN = :hidden
  ALSO_HIDDEN = :also_hidden

  private_constant(:HIDDEN, "ALSO_HIDDEN")

  def self.reach_hidden
    HIDDEN
  end
end

class Heir < Holder
end

puts(Holder::VISIBLE.inspect)
puts(Holder.reach_hidden.inspect)

begin
  Holder::HIDDEN
rescue NameError
  puts "HIDDEN is private"
end

begin
  Holder::ALSO_HIDDEN
rescue NameError
  puts "ALSO_HIDDEN is private"
end

begin
  Heir.send(:private_constant, :VISIBLE)
rescue NameError
  puts "NameError for an inherited constant"
end

begin
  Holder.send(:private_constant, :ABSENT)
rescue NameError
  puts "NameError for a missing constant"
end

Holder.send(:public_constant, :HIDDEN)
puts(Holder::HIDDEN.inspect)
