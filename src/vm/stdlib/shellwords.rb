# Splitting a command line the way a shell does, and writing a word back out
# so a shell reads it as one.
module Shellwords
  def self.shellsplit(line)
    words = []
    field = ""
    started = false
    rest = line
    until rest.empty?
      # The match is read out before anything else runs: a method called in
      # between would leave its own match behind.
      if rest =~ /\A\s+/
        rest = $'
        words.push(field) if started
        field = ""
        started = false
      elsif rest =~ /\A'([^']*)'/
        inner = $1
        rest = $'
        field += inner
        started = true
      elsif rest =~ /\A"((?:[^"\\]|\\.)*)"/
        inner = $1
        rest = $'
        field += inner.gsub(/\\([$`"\\\n])/, "\\1")
        started = true
      elsif rest =~ /\A\\(.)/m
        inner = $1
        rest = $'
        field += inner
        started = true
      elsif rest =~ /\A[^\s'"\\]+/
        inner = $&
        rest = $'
        field += inner
        started = true
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
