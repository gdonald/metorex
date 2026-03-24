# Query Builder DSL
# Demonstrates builder pattern for constructing SQL-like query strings

class QueryBuilder
  def initialize(table)
    @table = table
    @columns = ["*"]
    @conditions = []
    @order_field = nil
    @limit_count = nil
  end

  def select(cols)
    @columns = cols
    self
  end

  def where(condition)
    @conditions.push(condition)
    self
  end

  def order_by(field)
    @order_field = field
    self
  end

  def limit(n)
    @limit_count = n
    self
  end

  def to_sql
    col_str = @columns.join(", ")
    sql = "SELECT " + col_str + " FROM " + @table
    if @conditions.length > 0
      sql = sql + " WHERE " + @conditions.join(" AND ")
    end
    if @order_field
      sql = sql + " ORDER BY " + @order_field
    end
    if @limit_count
      sql = sql + " LIMIT #{@limit_count}"
    end
    sql
  end
end

q = QueryBuilder.new("users")
q.select(["name", "email"])
q.where("age > 18")
q.where("active = true")
q.order_by("name")
q.limit(10)
puts q.to_sql

q2 = QueryBuilder.new("products")
puts q2.to_sql
