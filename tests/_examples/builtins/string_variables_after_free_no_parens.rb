# A String given an instance variable and then dropped leaves nothing on the
# Strings made after it.
100.times do |count|
  marked = +"marked #{count}"
  marked.instance_variable_set :@count, count
end

carrying = 0
1000.times do |count|
  fresh = +"fresh #{count}"
  carrying += 1 unless fresh.instance_variables.empty?
end
puts carrying

list = [1]
list.instance_variable_set :@kept, true
puts list.instance_variables.inspect
