# nil, true and false report the methods of their own classes, and each of
# those methods is owned by that class.
[nil, true, false].each do |value|
  listed = value.class.instance_methods(false).sort
  p listed
  p listed.map { |name| value.method(name).owner }.uniq
end
p nil.method(:to_a).call
p true.method(:&).call(false)
p false.method(:inspect).unbind.bind(false).call
p nil.method(:object_id).owner
