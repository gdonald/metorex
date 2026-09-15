# `expect` reads a stream until what arrives matches a pattern, which is how a
# program driving another one waits for its prompt.

class IO
  # Read a character at a time until the whole of what has been read matches
  # `pattern`, answering the match and whatever it captured. The end of the
  # stream answers nothing. A block is handed the answer instead.
  def expect(pattern, _timeout = nil)
    wanted =
      case pattern
      when String then Regexp.new(Regexp.quote(pattern))
      when Regexp then pattern
      else raise TypeError, "unsupported pattern class: #{pattern.class}"
      end
    raise IOError, "closed stream" if closed?
    read_so_far = +''
    result = nil
    while (held = getc)
      read_so_far << held
      matched = wanted.match read_so_far
      next if matched.nil?
      result = [read_so_far] + matched.captures
      break
    end
    return yield result if block_given?
    result
  end
end
