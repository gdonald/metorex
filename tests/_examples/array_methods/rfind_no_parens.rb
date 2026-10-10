# `rfind` reads an array from the end, `Method#box` names no namespace while
# namespaces are off, and the core classes built natively descend from
# Object and through it from Kernel and BasicObject.
p [1, 2, 3].rfind { |number| number < 3 }
p [1, 2, 3].rfind(-> { :none }) { |number| number > 5 }
p [1, 2].rfind.to_a
shrinking = [1, 2, 3, 4]
p shrinking.rfind { shrinking.pop; false }

def answer = 42
p method(:answer).box
p 1.method(:+).box

p Time.ancestors
p Thread.ancestors
p Thread::Mutex.ancestors
