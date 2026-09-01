def caught
  yield
rescue NameError => error
  error.receiver
end

target = Object.new
puts caught { target.doesnt_exist }.equal? target
puts caught { DoesntExist }.equal? Object

module Namespace
  class Holder
  end
end

puts caught { Namespace::Holder::DoesntExist }.equal? Namespace::Holder

class WithoutTheVariable
  def read
    @@never_set
  end
end

puts caught { WithoutTheVariable.new.read }.equal? WithoutTheVariable
puts caught { target.instance_variable_get "bad name" }.equal? target
puts caught { Object.class_variable_get "bad name" }.equal? Object

begin
  NameError.new.receiver
rescue ArgumentError => error
  puts error.message
end
