# HTML Builder DSL
# Demonstrates method chaining and string building with OOP

class HtmlElement
  def initialize(tag, content)
    @tag = tag
    @content = content
    @attrs = ""
  end

  def set_attr(name, value)
    @attrs = @attrs + " " + name + "=\"" + value + "\""
    self
  end

  def to_html
    "<#{@tag}#{@attrs}>#{@content}</#{@tag}>"
  end
end

class HtmlBuilder
  def initialize
    @elements = []
  end

  def tag(name, content)
    el = HtmlElement.new(name, content)
    @elements.push(el)
    el
  end

  def heading(level, content)
    self.tag("h#{level}", content)
  end

  def paragraph(content)
    self.tag("p", content)
  end

  def span(content)
    self.tag("span", content)
  end

  def to_html
    result = ""
    @elements.each do |el|
      result = result + el.to_html + "\n"
    end
    result
  end
end

builder = HtmlBuilder.new
builder.heading(1, "Welcome to Metorex")
builder.paragraph("A meta-object programming language.").set_attr("class", "intro")
builder.span("Highlighted text").set_attr("class", "highlight").set_attr("id", "main")
puts builder.to_html
