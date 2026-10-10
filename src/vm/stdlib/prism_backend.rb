# frozen_string_literal: true

# What prism's C extension defines, written over the parser metorex compiles
# in. Prism serializes each result and prism's own Prism::Serialize reads it
# back, as prism/ffi.rb does.

module Prism
  VERSION = __prism_version__().freeze

  class << self
    def dump(source, **options)
      dumped = __prism_serialize__(:parse, prism_source(source), dump_options(options))
      dumped.freeze if options.fetch(:freeze, false)
      dumped
    end

    def dump_file(filepath, **options)
      options[:filepath] = filepath
      dump(prism_file(filepath), **options)
    end

    def lex(code, **options)
      serialized = __prism_serialize__(:lex, prism_source(code), dump_options(options))
      Serialize.load_lex(code, serialized, options.fetch(:freeze, false))
    end

    def lex_file(filepath, **options)
      options[:filepath] = filepath
      lex(prism_file(filepath), **options)
    end

    def parse(code, **options)
      Serialize.load_parse(code, dump(prism_source(code), **options), options.fetch(:freeze, false))
    end

    def parse_file(filepath, **options)
      options[:filepath] = filepath
      parse(prism_file(filepath), **options)
    end

    def parse_stream(stream, **options)
      source, serialized = __prism_serialize_stream__(stream, dump_options(options))
      Prism.load(source, serialized, options.fetch(:freeze, false))
    end

    def parse_comments(code, **options)
      serialized = __prism_serialize__(:parse_comments, prism_source(code), dump_options(options))
      Serialize.load_parse_comments(code, serialized, options.fetch(:freeze, false))
    end

    def parse_file_comments(filepath, **options)
      options[:filepath] = filepath
      parse_comments(prism_file(filepath), **options)
    end

    def parse_lex(code, **options)
      serialized = __prism_serialize__(:parse_lex, prism_source(code), dump_options(options))
      Serialize.load_parse_lex(code, serialized, options.fetch(:freeze, false))
    end

    def parse_lex_file(filepath, **options)
      options[:filepath] = filepath
      parse_lex(prism_file(filepath), **options)
    end

    def parse_success?(code, **options)
      __prism_parse_success__(prism_source(code), dump_options(options))
    end

    def parse_failure?(code, **options)
      !parse_success?(code, **options)
    end

    def parse_file_success?(filepath, **options)
      options[:filepath] = filepath
      parse_success?(prism_file(filepath), **options)
    end

    def parse_file_failure?(filepath, **options)
      !parse_file_success?(filepath, **options)
    end

    def profile(source, **options)
      dump(source, **options)
      nil
    end

    def profile_file(filepath, **options)
      dump_file(filepath, **options)
      nil
    end

    private

    def prism_source(source)
      raise TypeError, "wrong argument type #{source.class} (expected String)" unless source.is_a?(String)
      source
    end

    # The bytes of the file at `filepath`, read the way prism maps a file.
    def prism_file(filepath)
      raise TypeError, "no implicit conversion of #{filepath.class} into String" unless filepath.is_a?(String)
      raise Errno::EISDIR, filepath if File.directory?(filepath)
      File.binread(filepath)
    end

    # Return the value that should be dumped for the command_line option.
    def dump_options_command_line(options)
      command_line = options.fetch(:command_line, "")
      raise ArgumentError, "command_line must be a string" unless command_line.is_a?(String)

      command_line.each_char.inject(0) do |value, char|
        case char
        when "a" then value | 0b000001
        when "e" then value | 0b000010
        when "l" then value | 0b000100
        when "n" then value | 0b001000
        when "p" then value | 0b010000
        when "x" then value | 0b100000
        else raise ArgumentError, "invalid command_line option: #{char}"
        end
      end
    end

    # Return the value that should be dumped for the version option.
    def dump_options_version(version)
      current = version == "current"

      case current ? RUBY_VERSION : version
      when nil, "latest"
        0
      when /\A3\.3(\.\d+)?\z/
        1
      when /\A3\.4(\.\d+)?\z/
        2
      when /\A3\.5(\.\d+)?\z/, /\A4\.0(\.\d+)?\z/
        3
      when /\A4\.1(\.\d+)?\z/
        4
      else
        if current
          raise CurrentVersionError, RUBY_VERSION
        else
          raise ArgumentError, "invalid version: #{version}"
        end
      end
    end

    # Convert the given options into a serialized options string.
    def dump_options(options)
      template = +""
      values = []

      template << "L"
      if (filepath = options[:filepath])
        values.push(filepath.bytesize, filepath.b)
        template << "A*"
      else
        values << 0
      end

      template << "l"
      values << options.fetch(:line, 1)

      template << "L"
      if (encoding = options[:encoding])
        name = encoding.is_a?(Encoding) ? encoding.name : encoding
        values.push(name.bytesize, name.b)
        template << "A*"
      else
        values << 0
      end

      template << "C"
      values << (options.fetch(:frozen_string_literal, false) ? 1 : 0)

      template << "C"
      values << dump_options_command_line(options)

      template << "C"
      values << dump_options_version(options[:version])

      template << "C"
      values << (options[:encoding] == false ? 1 : 0)

      template << "C"
      values << (options.fetch(:main_script, false) ? 1 : 0)

      template << "C"
      values << (options.fetch(:partial_script, false) ? 1 : 0)

      template << "C"
      values << (options.fetch(:freeze, false) ? 1 : 0)

      template << "L"
      if (scopes = options[:scopes])
        values << scopes.length

        scopes.each do |scope|
          locals = nil
          forwarding = 0

          case scope
          when Array
            locals = scope
          when Scope
            locals = scope.locals

            scope.forwarding.each do |forward|
              case forward
              when :*     then forwarding |= 0x1
              when :**    then forwarding |= 0x2
              when :&     then forwarding |= 0x4
              when :"..." then forwarding |= 0x8
              else raise ArgumentError, "invalid forwarding value: #{forward}"
              end
            end
          else
            raise TypeError, "wrong argument type #{scope.class.inspect} (expected Array or Prism::Scope)"
          end

          template << "L"
          values << locals.length

          template << "C"
          values << forwarding

          locals.each do |local|
            name = local.name
            template << "L"
            values << name.bytesize

            template << "A*"
            values << name.b
          end
        end
      else
        values << 0
      end

      values.pack(template)
    end
  end

  class StringQuery
    class << self
      def local?(string)
        query(__prism_string_query__(:local, string, string.encoding.name))
      end

      def constant?(string)
        query(__prism_string_query__(:constant, string, string.encoding.name))
      end

      def method_name?(string)
        query(__prism_string_query__(:method_name, string, string.encoding.name))
      end

      private

      def query(result)
        case result
        when -1
          raise ArgumentError, "Invalid or non ascii-compatible encoding"
        when 0
          false
        when 1
          true
        end
      end
    end
  end
end
