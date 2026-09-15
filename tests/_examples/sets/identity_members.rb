# A set told to compare by identity keeps two equal strings apart, since they
# are different objects.
first = +"a"
second = first.dup

by_value = Set.new
by_value.merge [first, second]
p by_value.size

by_identity = Set.new.compare_by_identity
by_identity.merge [first, second]
p by_identity.size
p by_identity.compare_by_identity?
p by_value.compare_by_identity?
p by_identity.include? first
other = +"a"
p by_identity.include? other
p by_identity.dup.compare_by_identity?
p Set.new([1, 2]) == Set.new([1, 2]).compare_by_identity

begin
  Set.new.freeze.compare_by_identity
rescue FrozenError => problem
  puts problem.class
end
