class Clause
  def in = "in"
  def then = "then"
  def do = "do"
  def redo = "redo"
  def retry = "retry"
  def and = "and"
  def or = "or"
  def not = "not"
  def unless = "unless"
end

clause = Clause.new
p [clause.in, clause.then, clause.do, clause.redo, clause.retry]
p [clause.and, clause.or, clause.not, clause.unless]

class Amount
  def initialize cents
    @cents = cents
  end

  def matches? other
    false
  end
  def present? = true
end

class Invoice < Amount
  def matches? other
    super or other == 100
  end
  def present?; super and @cents.positive?; end
end

p Invoice.new(100).matches? 100
p Invoice.new(100).matches? 5
p Invoice.new(0).present?
