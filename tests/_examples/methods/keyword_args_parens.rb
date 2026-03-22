def greet(name:, greeting: "Hello")
  puts("#{greeting}, #{name}!")
end

greet(name: "Alice")
greet(name: "Bob", greeting: "Hi")
greet(greeting: "Hey", name: "Carol")
