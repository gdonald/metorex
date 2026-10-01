# The unambiguous beginnings of a set of words, which is what a command line
# uses to let a name be typed short.
module Abbrev
  # Each word's abbreviations, longest first, are kept while only that word
  # has them; one two words share is dropped, and every word stands for
  # itself. A String pattern keeps only what starts with it, and any other
  # pattern only what it matches.
  def self.abbrev(words, pattern = nil)
    table = {}
    seen = Hash.new(0)
    pattern = /\A#{Regexp.quote(pattern)}/ if pattern.is_a?(String)

    words.each do |word|
      next if word.empty?
      word.size.downto(1) do |length|
        abbreviation = word[0...length]
        next if pattern && pattern !~ abbreviation

        case seen[abbreviation] += 1
        when 1
          table[abbreviation] = word
        when 2
          table.delete(abbreviation)
        else
          break
        end
      end
    end

    words.each do |word|
      next if pattern && pattern !~ word

      table[word] = word
    end

    table
  end
end

class Array
  def abbrev(pattern = nil)
    Abbrev.abbrev(self, pattern)
  end
end
