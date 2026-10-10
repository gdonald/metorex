# The object a program runs against at the top level carries define_method,
# include, private, public, ruby2_keywords and using as private methods of
# its singleton class, which `send` reaches the way the bare forms run.
main = TOPLEVEL_BINDING.receiver
p(main.private_methods(false).sort)
p(main.singleton_methods.sort)
p(main.singleton_class.private_instance_methods(false).sort)
p([main.respond_to?(:define_method), main.respond_to?(:define_method, true)])

main.send(:define_method, :greeting) { :hello }
p(greeting)
main.send(:private, :greeting)
p(main.respond_to?(:greeting))
main.send(:public, :greeting)
p(main.respond_to?(:greeting))

module Shouting
  def shout = :loud
end
main.send(:include, Shouting)
p([Object.include?(Shouting), shout])

module Upcased
  refine(String) { def upcased = upcase }
end
main.send(:using, Upcased)
p("quiet".upcased)

begin
  main.define_method(:other) { 1 }
rescue NoMethodError => error
  p(error.message)
end
