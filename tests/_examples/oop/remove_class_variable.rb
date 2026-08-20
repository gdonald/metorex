module Vars
  @@shared = :shared
end

class Holder
  @@own = :own
end

copy = Vars.dup
puts copy.send(:remove_class_variable, :@@shared).inspect
puts copy.class_variable_defined?(:@@shared).inspect

puts Holder.send(:remove_class_variable, "@@own").inspect

class Includer
  include Vars
end

[:@@shared, :@shared, :shared, :@@absent].each do |name|
  begin
    Includer.send(:remove_class_variable, name)
    puts "#{name}: removed"
  rescue NameError
    puts "#{name}: NameError"
  end
end

puts Module.private_instance_methods.include?(:remove_class_variable).inspect

def reports_block &block
  block.nil? ? :no_block : :got_block
end

Holder.new { :ignored }
puts reports_block().inspect
