def report
  yield
rescue NameError => error
  p error.name
end

report { doesnt_exist }
report { DoesntExist }
report { Object::DoesntExist }

ivar_name = "invalid_ivar_name"
report { Object.new.instance_variable_get ivar_name }

cvar_name = "invalid_cvar_name"
report { Object.class_variable_get cvar_name }

class Counter
  @@total = 7

  def self.total
    @@total
  end

  def self.missing
    @@never_set
  end
end

class Subcounter < Counter
end

puts Counter.total
puts Subcounter.total

begin
  Counter.missing
rescue NameError => error
  puts error.message
  p error.name
end
