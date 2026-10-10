# Code inside `#{}` and code given to `eval` is read knowing the locals of
# the scope it runs in, so `name [1]` indexes a local named `name` rather
# than calling a method of that name with an Array.
def items(*arguments)
  [:method, arguments]
end

p("#{items [1]}")
items = [10, 20]
p("#{items [1]}")
p(:"#{items [0]}")
p(/#{items [0]}/)
[3].each { |number| p("#{number [0]}") }

p(eval("items [1]"))
p(binding.eval("items [0]"))
p(Object.new.instance_eval("items [1]"))
p(Class.new.class_eval("items [0]"))

def scoped
  items = [3, 4]
  binding
end
p(eval("items [1]", scoped))

def method_scope
  "#{items [1]}"
end
p(method_scope)
