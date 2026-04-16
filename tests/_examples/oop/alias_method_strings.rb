class Foo
  def original
    "original"
  end

  alias_method "aliased", "original"
  alias_method :sym_alias, :original
end

f = Foo.new
puts f.aliased
puts f.sym_alias
