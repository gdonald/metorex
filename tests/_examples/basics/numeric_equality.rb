puts 1 == 1.0
puts 1.0 == 1
puts 1 == 1.5
puts 2 != 2.0
puts([1, 2] == [1.0, 2.0])
puts({a: 1} == {a: 1.0})

class Probe
  def to_s
    "probe"
  end
end

probe = Probe.new
defined_to_s = defined?(probe.to_s)
p defined_to_s
defined_missing = defined?(probe.missing_method)
p defined_missing
defined_new = defined?(String.new)
p defined_new
defined_absent = defined?(String.no_such_method)
p defined_absent
