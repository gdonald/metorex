class Keywords
  def meth
    :original
  end

  alias_method :alias, :meth
  alias_method :until, :meth
  alias_method :extend, :meth
end

subject = Keywords.new
puts subject.alias.inspect
puts subject.until.inspect
puts subject.extend.inspect

def take first, second
  [first, second]
end

pair = take :alias, :meth
puts pair.inspect
