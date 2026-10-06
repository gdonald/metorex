# extend takes several modules and puts the first one named nearest the
# object, and refuses anything that is not a module.
module Loud
  def speak = :loud
end

module Quiet
  def speak = :quiet
end

class Speaker
  extend(Loud, Quiet)
end

p(Speaker.speak)
p(Speaker.singleton_class.ancestors.take(3))
voice = Object.new
voice.extend(Quiet, Loud)
p(voice.speak)
[[1], [], [Speaker]].each do |given|
  Speaker.extend(*given)
rescue TypeError, ArgumentError => error
  p([error.class, error.message])
end
