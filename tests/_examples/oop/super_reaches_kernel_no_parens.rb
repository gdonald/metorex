class Loader
  def require_relative(path)
    super
  end

  def frozen?
    super
  end

  def inspect
    "loader: #{super}"
  end

  def instance_variable_get(name)
    super name
  end
end

loader = Loader.new
loader.instance_variable_set :@state, :ready

puts loader.frozen?
state = loader.instance_variable_get :@state
puts state.inspect
puts loader.inspect.start_with? "loader: #<Loader"

class Counter
  def itself
    super
  end
end

counter = Counter.new
puts counter.itself.equal? counter

class Missing
  def no_such_kernel_method
    super
  end
end

begin
  Missing.new.no_such_kernel_method
rescue RuntimeError => error
  puts error.message
end
