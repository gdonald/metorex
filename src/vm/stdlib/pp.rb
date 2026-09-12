# Printing an object the way `pp` does, written out through a stream the
# caller names.
module PP
  def self.pp(held, out = $stdout, width = 79)
    out.write(held.pretty_inspect)
    out
  end

  def self.singleline_pp(held, out = $stdout)
    out.write(held.inspect)
    out
  end
end
