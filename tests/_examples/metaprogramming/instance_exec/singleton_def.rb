# A `def` written at the top of an instance_exec block belongs to the receiver
# alone, so another instance of the same class does not answer it.
one = Object.new

one.instance_exec do
  def greeting
    "hello"
  end
end

puts one.greeting
puts one.singleton_methods.inspect
puts Object.new.respond_to?(:greeting)

counter = Object.new
counter.instance_eval do
  def next_value
    @value = @value.to_i + 1
  end
end

puts counter.next_value
puts counter.next_value
