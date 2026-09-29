# Finalizers run as the program ends, including one defined while they run.
class Resource
  def initialize(name)
    @name = name
  end
end

first = Resource.new("first")
handler = proc { |id| puts("first finalized") }
p(ObjectSpace.define_finalizer(first, handler) == [0, handler])
# An equal finalizer is kept once, and the one already there is answered.
p(ObjectSpace.define_finalizer(first, handler.dup())[1].equal?(handler))

second = Resource.new("second")
ObjectSpace.define_finalizer(second, proc do |id|
  third = Resource.new("third")
  ObjectSpace.define_finalizer(third, proc { |inner| puts("third finalized") })
  puts("second finalized")
end)

# A finalizer that raises is reported, and the others still run.
ObjectSpace.define_finalizer(Resource.new("failing"), proc { |id| raise("finalizing") })

begin
  ObjectSpace.define_finalizer(:name) { }
rescue ArgumentError => error
  p(error.message)
end
begin
  ObjectSpace.define_finalizer(first, Object.new)
rescue ArgumentError => error
  p(error.message)
end
