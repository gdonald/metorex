module Holder
  KEPT = :kept
  DOOMED = :doomed
  ALSO = :also
end

class Heir
  include Holder
end

class NameSource
  def to_str
    "ALSO"
  end
end

puts Holder.send(:remove_const, :DOOMED).inspect
puts Holder.send(:remove_const, NameSource.new).inspect
puts Holder.constants.inspect

["name", "__CONSTX__", "@Name", "Name=", "Missing"].each do |name|
  begin
    Holder.send(:remove_const, name)
    puts "#{name}: removed"
  rescue NameError
    puts "#{name}: NameError"
  end
end

begin
  Heir.send(:remove_const, :KEPT)
rescue NameError
  puts "inherited: NameError"
end

module Lazy
  autoload :Later, "a_file"
end
puts Lazy.send(:remove_const, :Later).inspect

puts Module.private_methods.include?(:remove_const).inspect
puts Module.public_instance_methods.include?(:alias_method).inspect
