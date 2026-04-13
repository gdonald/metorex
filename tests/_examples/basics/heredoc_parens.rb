# Heredoc examples (method calls with parens)
text = <<~HEREDOC
  Hello, World!
  This is a heredoc.
HEREDOC
puts(text)

greeting = <<~MSG
  Good morning!
MSG
puts(greeting.strip)
