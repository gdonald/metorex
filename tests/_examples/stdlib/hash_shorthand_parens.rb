# Hash shorthand syntax with parens: {key: value}
person = {name: "Alice", age: 30}
puts(person[:name])
puts(person[:age])

config = {host: "localhost", port: 8080, debug: true}
puts(config[:host])
puts(config[:port])
puts(config[:debug])

mixed = {a: 1, b: 2, c: 3}
puts(mixed.length)
puts(mixed[:b])
