# The unambiguous beginnings of a set of words, which is what a command line
# uses to let a name be typed short.
module Abbrev
  def self.abbrev(words, pattern = nil)
    seen = Hash.new(0)
    table = {}
    words.each do |word|
      next if word.empty?
      length = word.length
      (1...length).each do |cut|
        prefix = word[0, cut]
        seen[prefix] += 1
      end
    end
    words.each do |word|
      next if word.empty?
      length = word.length
      (1...length).each do |cut|
        prefix = word[0, cut]
        table[prefix] = word if seen[prefix] == 1
      end
      table[word] = word
    end
    return table if pattern.nil?
    picked = {}
    table.each { |prefix, word| picked[prefix] = word if pattern === prefix }
    picked
  end
end

class Array
  def abbrev(pattern = nil)
    Abbrev.abbrev(self, pattern)
  end
end
