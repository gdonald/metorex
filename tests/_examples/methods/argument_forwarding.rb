def target(a, b:, &block)
  [a, b, block.call]
end

def forwards_all(...)
  target(...)
end

p forwards_all(1, b: 2) { 3 }

def positional_target(a, b, c)
  [a, b, c]
end

def forwards_rest(*)
  positional_target(*)
end

p forwards_rest(1, 2, 3)

def keyword_target(a:, b:)
  [a, b]
end

def forwards_keywords(**)
  keyword_target(**)
end

p forwards_keywords(a: 1, b: 2)

def block_target(&block)
  block.call
end

def forwards_block(&)
  block_target(&)
end

p forwards_block { :done }

p 7.clamp(..5)
