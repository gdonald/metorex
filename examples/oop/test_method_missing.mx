class DynamicProxy
  def initialize
    @data = {"foo" => "bar", "count" => 42, "items" => [1, 2, 3]}
  end

  def method_missing(name)
    @data[name]
  end
end

proxy = DynamicProxy.new
puts proxy.foo
puts proxy.count

items = proxy.items
items.each do |item|
  puts item
end
