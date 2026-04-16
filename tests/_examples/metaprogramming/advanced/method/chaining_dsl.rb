# Method Chaining DSL: Build fluent interfaces
# Demonstrates: returning self, builder pattern, domain-specific languages

class QueryBuilder
  def initialize
    @table = nil
    @conditions = []
    @order = nil
    @limit_val = nil
  end

  def from(table)
    @table = table
    self
  end

  def where(condition)
    @conditions.push(condition)
    self
  end

  def order_by(field)
    @order = field
    self
  end

  def limit(n)
    @limit_val = n
    self
  end

  def to_sql
    sql = "SELECT * FROM #{@table}"
    if @conditions.length > 0
      parts = @conditions.join(" AND ")
      sql = "#{sql} WHERE #{parts}"
    end
    if @order != nil
      sql = "#{sql} ORDER BY #{@order}"
    end
    if @limit_val != nil
      sql = "#{sql} LIMIT #{@limit_val}"
    end
    sql
  end
end

puts "=== Query Builder DSL ==="

query1 = QueryBuilder.new.from("users").where("age > 18").where("active = true").order_by("name").limit(10)
puts query1.to_sql

query2 = QueryBuilder.new.from("products").where("price < 100")
puts query2.to_sql

# HTML Builder DSL
class HtmlBuilder
  def initialize
    @parts = []
  end

  def tag(name, content)
    @parts.push("<#{name}>#{content}</#{name}>")
    self
  end

  def heading(text)
    self.tag("h1", text)
  end

  def paragraph(text)
    self.tag("p", text)
  end

  def bold(text)
    self.tag("b", text)
  end

  def to_html
    @parts.join("\n")
  end
end

puts ""
puts "=== HTML Builder DSL ==="

html = HtmlBuilder.new.heading("Welcome").paragraph("This is a paragraph.").bold("Important!").paragraph("Another paragraph.")
puts html.to_html

# Configuration DSL
class Config
  def initialize
    @settings = {}
  end

  def set(key, value)
    @settings[key] = value
    self
  end

  def display
    @settings.each do |key, value|
      puts "  #{key} = #{value}"
    end
  end
end

puts ""
puts "=== Configuration DSL ==="

config = Config.new.set("host", "localhost").set("port", "8080").set("debug", "true")
puts "Settings:"
config.display
