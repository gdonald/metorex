# Splitting a command line the way a shell does, and writing a word back out
# so a shell reads it as one.
module Shellwords
  def self.shellsplit(line)
    words = []
    field = ""
    started = false
    rest = line
    until rest.empty?
      if rest =~ /\A\s+/
        words.push(field) if started
        field = ""
        started = false
        rest = $'
      elsif rest =~ /\A'([^']*)'/
        field += $1
        started = true
        rest = $'
      elsif rest =~ /\A"((?:[^"\\]|\\.)*)"/
        field += $1.gsub(/\\([$`"\\\n])/, "\\1")
        started = true
        rest = $'
      elsif rest =~ /\A\\(.)/m
        field += $1
        started = true
        rest = $'
      elsif rest =~ /\A[^\s'"\\]+/
        field += $&
        started = true
        rest = $'
      else
        raise ArgumentError, "Unmatched quote: #{line.inspect}"
      end
    end
    words.push(field) if started
    words
  end

  def self.shellwords(line)
    shellsplit(line)
  end

  def self.shellescape(word)
    text = word.to_s
    return "''" if text.empty?
    escaped = ""
    text.each_char do |letter|
      if letter =~ /[A-Za-z0-9_\-.,:+\/@]/
        escaped += letter
      elsif letter == "\n"
        escaped += "'\n'"
      else
        escaped += "\\" + letter
      end
    end
    escaped
  end

  def self.escape(word)
    shellescape(word)
  end

  def self.shelljoin(words)
    words.map { |word| shellescape(word) }.join(" ")
  end

  def self.join(words)
    shelljoin(words)
  end

  def self.split(line)
    shellsplit(line)
  end
end

class String
  def shellsplit
    Shellwords.shellsplit(self)
  end

  def shellescape
    Shellwords.shellescape(self)
  end
end

class Array
  def shelljoin
    Shellwords.shelljoin(self)
  end
end
