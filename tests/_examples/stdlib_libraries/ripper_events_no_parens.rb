require 'ripper'

source = "def area(width, height = 2) = width * height\nputs area(3)\n"

p Ripper.tokenize(source)
Ripper.lex("total = price * 2").each { |position, event, token, state| p([position, event, token, state.to_s]) }
p Ripper.sexp("total = price * 2")
p Ripper.sexp_raw("[1, *rest]")
p Ripper.sexp(source)
p Ripper.sexp("def broken(")
p Ripper.lex_state_name(Ripper::EXPR_BEG | Ripper::EXPR_LABEL)

heredoc = "message = <<~TEXT\n    Deploy finished\n      on schedule\nTEXT\n"
p Ripper.sexp(heredoc)
p Ripper.lex(heredoc).map { |_position, event, token| [event, token] }

p Ripper.sexp("case reading\nin {status: 200, body:} then body\nin [Integer => code, *] then code\nend")

# A subclass sees each event through the method named for it.
class MethodNames < Ripper
  def on_def(name, _params, _body)
    (@names ||= []) << name
    name
  end

  def on_ident(token) = token

  def names
    parse
    @names
  end
end
p MethodNames.new("def start; end\ndef stop = nil\n").names

# A filter walks the tokens in order, threading a value through them.
class CommentCounter < Ripper::Filter
  def on_comment(token, count) = count + 1

  def on_default(_event, _token, count) = count
end
p CommentCounter.new("# setup\nvalue = 1 # inline\n").parse(0)

p Ripper.slice("print(1 + 2)", "ident")
p Ripper.dedent_string(+"    indented", 2)
