class Namespace < Module
  attr_reader :label

  def initialize
    @label = :named
  end
end

space = Namespace.new
puts(space.label.inspect)
puts(space.class.inspect)
puts(space.is_a?(Module).inspect)
puts(space.constants.inspect)

space.const_set(:LIMIT, 10)
puts(space.constants.inspect)
puts(space.const_get(:LIMIT).inspect)

blank = Module.new do
  const_set(:A, 'A')
end
puts(blank.const_get('A').inspect)
