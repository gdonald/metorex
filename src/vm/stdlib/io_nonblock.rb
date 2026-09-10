# Whether reads and writes on a stream come back straight away rather than
# waiting. Metorex opens every socket that way and every file the other.

class IO
  def nonblock?
    @nonblock.nil? ? false : @nonblock
  end

  def nonblock= flag
    @nonblock = flag
  end

  def nonblock flag = true
    unless block_given?
      self.nonblock = flag
      return flag
    end
    held = nonblock?
    self.nonblock = flag
    begin
      yield self
    ensure
      self.nonblock = held
    end
  end
end
