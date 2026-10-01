# Ripper.sexp and Ripper.sexp_raw: the parse as nested arrays, each parser
# event a symbol followed by its arguments and each scanner event its
# token with its position.

require 'ripper/core'

class Ripper
  def self.sexp(src, filename = "-", lineno = 1, raise_errors: false)
    builder = SexpBuilderPP.new(src, filename, lineno)
    sexp = builder.parse
    if builder.error?
      raise SyntaxError, builder.error if raise_errors
    else
      sexp
    end
  end

  def self.sexp_raw(src, filename = "-", lineno = 1, raise_errors: false)
    builder = SexpBuilder.new(src, filename, lineno)
    sexp = builder.parse
    if builder.error?
      raise SyntaxError, builder.error if raise_errors
    else
      sexp
    end
  end

  class SexpBuilder < ::Ripper
    attr_reader :error

    private

    def dedent_element(element, width)
      removed = dedent_string(element[1], width)
      element[2][1] += removed if removed.positive?
      element
    end

    def on_heredoc_dedent(value, width)
      dedent = proc do |contents|
        contents.map! do |element|
          if element.is_a?(Array)
            case element[0]
            when :@tstring_content
              element = dedent_element(element, width)
            when /_add\z/
              element[1] = dedent[element[1]]
            end
          elsif element.is_a?(String)
            dedent_string(element, width)
          end
          element
        end
      end
      dedent[value]
      value
    end

    defined_here = private_instance_methods(false).grep(/\Aon_/) { $'.to_sym }
    (PARSER_EVENTS - defined_here).each do |event|
      define_method(:"on_#{event}") { |*args| args.unshift(event) }
    end

    SCANNER_EVENTS.each do |event|
      define_method(:"on_#{event}") { |token| [:"@#{event}", token, [lineno, column]] }
    end

    def on_error(message)
      @error = message
    end

    alias on_parse_error on_error
    alias compile_error on_error
  end

  # SexpBuilder with each list event collapsed into a plain array.
  class SexpBuilderPP < SexpBuilder
    private

    def on_heredoc_dedent(value, width)
      value.map! do |element|
        next element if element.is_a?(Symbol) && element.to_s.end_with?("_content")

        if element.is_a?(Array) && element[0] == :@tstring_content
          element = dedent_element(element, width)
        elsif element.is_a?(String)
          dedent_string(element, width)
        end
        element
      end
      value
    end

    def _dispatch_event_new = []

    def _dispatch_event_push(list, item)
      list.push(item)
      list
    end

    def on_mlhs_paren(list) = [:mlhs, *list]

    def on_mlhs_add_star(list, star) = list.push([:rest_param, star])

    def on_mlhs_add_post(list, post) = list.concat(post)

    PARSER_EVENT_TABLE.each do |event, arity|
      if event.to_s.end_with?("_new") && arity.zero?
        alias_method "on_#{event}", :_dispatch_event_new
      elsif event.to_s.end_with?("_add")
        alias_method "on_#{event}", :_dispatch_event_push
      end
    end
  end
end
