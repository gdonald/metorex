# `Warning.categories` lists every category `Warning[]` accepts.
p Warning.categories

# :experimental starts on, the rest start off.
p Warning[:deprecated]
p Warning[:experimental]
p Warning[:performance]

Warning[:performance] = true
p Warning[:performance]
Warning[:performance] = false

begin
  Warning[:noop]
rescue ArgumentError => error
  puts error.message
end

begin
  Warning["deprecated"]
rescue TypeError => error
  puts error.message
end

begin
  Warning[42] = true
rescue TypeError => error
  puts error.message
end

# Warning extends itself, so `warn` is an instance method it answers to.
p Warning.singleton_class.ancestors.include? Warning
p Warning.method(:warn).owner

# A warning in a category that is switched off never prints.
Warning[:deprecated] = false
p Warning.warn("suppressed\n", category: :deprecated)
Warning[:deprecated] = true
Warning.warn "printed\n", category: :deprecated

# A key written twice in one literal is named once, and the warning travels
# through Warning.warn, which a program can replace.
$VERBOSE = false
$collected = []

def Warning.warn(message, category: nil)
  $collected << message
  nil
end

duplicated = { key: :value, key: :value2 }
p duplicated
p $collected.size
p $collected.first.include? "key :key is duplicated and overwritten"
