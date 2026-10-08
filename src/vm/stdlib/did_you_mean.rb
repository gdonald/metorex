# The suggestions Ruby appends to the report of a NameError, NoMethodError,
# KeyError, LoadError or NoMatchingPatternKeyError, found by spelling the
# missed name against the names that were there.
require "rbconfig"

module DidYouMean
  VERSION = "2.0.0".freeze

  # How far apart two words are as the number of single-character edits
  # between them.
  module Levenshtein # :nodoc:
    def distance(str1, str2)
      n = str1.length
      m = str2.length
      return m if n.zero?
      return n if m.zero?

      d = (0..m).to_a
      x = nil
      str2_codepoints = str2.codepoints
      str1.each_codepoint.with_index(1) do |char1, i|
        j = 0
        while j < m
          cost = (char1 == str2_codepoints[j]) ? 0 : 1
          x = min3(
            d[j + 1] + 1,
            i + 1,
            d[j] + cost
          )
          d[j] = i
          i = x
          j += 1
        end
        d[m] = x
      end
      x
    end
    module_function :distance

    private

    def min3(a, b, c)
      if a < b && a < c
        a
      elsif b < c
        b
      else
        c
      end
    end
    module_function :min3
  end

  # How alike two words are, from 0 to 1, by the characters they share near
  # the same places.
  module Jaro
    module_function

    def distance(str1, str2)
      str1, str2 = str2, str1 if str1.length > str2.length
      length1, length2 = str1.length, str2.length

      m = 0.0
      t = 0.0
      range = length2 > 3 ? length2 / 2 - 1 : 0
      flags1 = 0
      flags2 = 0

      str1_codepoints = str1.codepoints
      str2_codepoints = str2.codepoints

      i = 0
      while i < length1
        last = i + range
        j = (i >= range) ? i - range : 0
        while j <= last
          if flags2[j] == 0 && str1_codepoints[i] == str2_codepoints[j]
            flags2 |= (1 << j)
            flags1 |= (1 << i)
            m += 1
            break
          end
          j += 1
        end
        i += 1
      end

      k = i = 0
      while i < length1
        if flags1[i] != 0
          j = index = k
          k = while j < length2
                index = j
                break(j + 1) if flags2[j] != 0
                j += 1
              end
          t += 1 if str1_codepoints[i] != str2_codepoints[index]
        end
        i += 1
      end
      t = (t / 2).floor

      m == 0 ? 0 : (m / length1 + m / length2 + (m - t) / m) / 3
    end
  end

  # The Jaro distance raised for words that start the same way.
  module JaroWinkler
    WEIGHT = 0.1
    THRESHOLD = 0.7

    module_function

    def distance(str1, str2)
      jaro_distance = Jaro.distance(str1, str2)

      if jaro_distance > THRESHOLD
        codepoints2 = str2.codepoints
        prefix_bonus = 0
        str1.each_codepoint do |char1|
          char1 == codepoints2[prefix_bonus] && prefix_bonus < 4 ? prefix_bonus += 1 : break
        end
        jaro_distance + (prefix_bonus * WEIGHT * (1 - jaro_distance))
      else
        jaro_distance
      end
    end
  end

  class SpellChecker
    def initialize(dictionary:)
      @dictionary = dictionary
    end

    def correct(input)
      normalized_input = normalize(input)
      threshold = normalized_input.length > 3 ? 0.834 : 0.77
      words = @dictionary.select { |word| JaroWinkler.distance(normalize(word), normalized_input) >= threshold }
      words.reject! { |word| input.to_s == word.to_s }
      words.sort_by! { |word| JaroWinkler.distance(word.to_s, normalized_input) }
      words.reverse!

      # Correct mistypes
      threshold = (normalized_input.length * 0.25).ceil
      corrections = words.select { |c| Levenshtein.distance(normalize(c), normalized_input) <= threshold }

      # Correct misspells
      if corrections.empty?
        corrections = words.select do |word|
          word = normalize(word)
          length = normalized_input.length < word.length ? normalized_input.length : word.length

          Levenshtein.distance(word, normalized_input) < length
        end.first(1)
      end

      corrections
    end

    private

    def normalize(str_or_symbol) #:nodoc:
      str = str_or_symbol.to_s.downcase
      str.tr!("@", "")
      str
    end
  end

  # A spell checker for paths, which corrects each directory of the path
  # against the directories the dictionary holds at that depth.
  class TreeSpellChecker
    attr_reader :dictionary, :separator, :augment

    def initialize(dictionary:, separator: "/", augment: nil)
      @dictionary = dictionary
      @separator = separator
      @augment = augment
    end

    def correct(input)
      plausibles = plausible_dimensions(input)
      return fall_back_to_normal_spell_check(input) if plausibles.empty?

      suggestions = find_suggestions(input, plausibles)
      return fall_back_to_normal_spell_check(input) if suggestions.empty?

      suggestions
    end

    def dictionary_without_leaves
      @dictionary_without_leaves ||= dictionary.map { |word| word.split(separator)[0..-2] }.uniq
    end

    def tree_depth
      @tree_depth ||= dictionary_without_leaves.max { |a, b| a.size <=> b.size }.size
    end

    def dimensions
      @dimensions ||= tree_depth.times.map do |index|
        dictionary_without_leaves.map { |element| element[index] }.compact.uniq
      end
    end

    def find_leaves(path)
      path_with_separator = "#{path}#{separator}"

      dictionary
        .select { |str| str.include?(path_with_separator) }
        .map { |str| str.gsub(path_with_separator, "") }
    end

    def plausible_dimensions(input)
      input.split(separator)[0..-2]
        .map
        .with_index { |element, index| correct_element(dimensions[index], element) if dimensions[index] }
        .compact
    end

    def possible_paths(states)
      states.map { |state| state.join(separator) }
    end

    private

    def find_suggestions(input, plausibles)
      states = plausibles[0].product(*plausibles[1..-1])
      paths = possible_paths(states)
      leaf = input.split(separator).last

      find_ideas(paths, leaf)
    end

    def fall_back_to_normal_spell_check(input)
      return [] unless augment

      ::DidYouMean::SpellChecker.new(dictionary: dictionary).correct(input)
    end

    def find_ideas(paths, leaf)
      paths.flat_map do |path|
        names = find_leaves(path)
        ideas = correct_element(names, leaf)

        ideas_to_paths(ideas, leaf, names, path)
      end.compact
    end

    def ideas_to_paths(ideas, leaf, names, path)
      if ideas.empty?
        nil
      elsif names.include?(leaf)
        ["#{path}#{separator}#{leaf}"]
      else
        ideas.map { |str| "#{path}#{separator}#{str}" }
      end
    end

    def correct_element(names, element)
      return names if names.size == 1

      str = normalize(element)

      return [str] if names.include?(str)

      ::DidYouMean::SpellChecker.new(dictionary: names).correct(str)
    end

    def normalize(str)
      str.downcase!
      str.tr!("@", " ") if str.include?("@")
      str
    end
  end

  # Mixed into each error class did_you_mean corrects, ahead of its own
  # methods, so the report of the error carries the suggestions.
  module Correctable
    def original_message
      to_s
    end

    def detailed_message(highlight: true, did_you_mean: true, **)
      msg = super.dup

      return msg unless did_you_mean

      suggestion = DidYouMean.formatter.message_for(corrections)

      if highlight
        suggestion = suggestion.gsub(/.+/) { "\e[1m" + $& + "\e[m" }
      end

      msg << suggestion
      msg
    rescue
      super
    end

    def corrections
      @corrections ||= spell_checker.corrections
    end

    def spell_checker
      DidYouMean.spell_checkers[self.class.to_s].new(self)
    end
  end

  class NullChecker
    def initialize(*); end

    def corrections
      []
    end
  end

  class ClassNameChecker
    attr_reader :class_name

    def initialize(exception)
      @class_name, @receiver, @original_message = exception.name, exception.receiver, exception.original_message
    end

    def corrections
      @corrections ||= SpellChecker.new(dictionary: class_names)
                         .correct(class_name)
                         .map(&:full_name)
                         .reject { |qualified_name| @original_message.include?(qualified_name) }
    end

    def class_names
      scopes.flat_map do |scope|
        scope.constants.map do |c|
          ClassName.new(c, scope == Object ? "" : "#{scope}::")
        end
      end
    end

    def scopes
      @scopes ||= @receiver.to_s.split("::").inject([Object]) do |_scopes, scope|
        _scopes << _scopes.last.const_get(scope)
      end.uniq
    end

    class ClassName < String
      attr :namespace

      def initialize(name, namespace = "")
        super(name.to_s)
        @namespace = namespace
      end

      def full_name
        self.class.new("#{namespace}#{self}")
      end
    end

    private_constant :ClassName
  end

  class VariableNameChecker
    attr_reader :name, :method_names, :lvar_names, :ivar_names, :cvar_names

    NAMES_TO_EXCLUDE = { "foo" => [:fork, :for] }
    NAMES_TO_EXCLUDE.default = []

    # The keywords a misspelled name may have meant.
    RB_RESERVED_WORDS = %i(
      BEGIN
      END
      alias
      and
      begin
      break
      case
      class
      def
      defined?
      else
      elsif
      end
      ensure
      false
      for
      module
      next
      nil
      not
      redo
      rescue
      retry
      return
      self
      super
      then
      true
      undef
      unless
      until
      when
      while
      yield
      __LINE__
      __FILE__
      __ENCODING__
    )

    def initialize(exception)
      @name = exception.name.to_s.tr("@", "")
      @lvar_names = exception.respond_to?(:local_variables) ? exception.local_variables : []
      receiver = exception.receiver

      @method_names = receiver.methods + receiver.private_methods
      @ivar_names = receiver.instance_variables
      @cvar_names = receiver.class.class_variables
      @cvar_names += receiver.class_variables if receiver.kind_of?(Module)
    end

    def corrections
      @corrections ||= SpellChecker
                         .new(dictionary: (RB_RESERVED_WORDS + lvar_names + method_names + ivar_names + cvar_names))
                         .correct(name).uniq - NAMES_TO_EXCLUDE[@name]
    end
  end

  class << (NameErrorCheckers = Object.new)
    def new(exception)
      case exception.original_message
      when /uninitialized constant/
        ClassNameChecker
      when /undefined local variable or method/,
           /undefined method/,
           /uninitialized class variable/,
           /no member '.*' in struct/
        VariableNameChecker
      else
        NullChecker
      end.new(exception)
    end
  end

  class MethodNameChecker
    attr_reader :method_name, :receiver

    NAMES_TO_EXCLUDE = { NilClass => nil.methods }
    NAMES_TO_EXCLUDE.default = []

    # The keywords a misspelled call without a receiver may have meant.
    RB_RESERVED_WORDS = %i(
      alias
      case
      def
      defined?
      elsif
      end
      ensure
      for
      rescue
      super
      undef
      unless
      until
      when
      while
      yield
    )

    def initialize(exception)
      @method_name = exception.name
      @receiver = exception.receiver
      @private_call = exception.respond_to?(:private_call?) ? exception.private_call? : false
    end

    def corrections
      @corrections ||= begin
        dictionary = method_names
        dictionary = RB_RESERVED_WORDS + dictionary if @private_call

        SpellChecker.new(dictionary: dictionary).correct(method_name) - names_to_exclude
      end
    end

    def method_names
      if Object === receiver
        method_names = receiver.methods + receiver.singleton_methods
        method_names += receiver.private_methods if @private_call
        method_names.uniq!
        # Assume that people trying to use a writer are not interested in a reader
        # and vice versa
        if method_name.match?(/=\Z/)
          method_names.select! { |name| name.match?(/=\Z/) }
        else
          method_names.reject! { |name| name.match?(/=\Z/) }
        end
        method_names
      else
        []
      end
    end

    def names_to_exclude
      Object === receiver ? NAMES_TO_EXCLUDE[receiver.class] : []
    end
  end

  class KeyErrorChecker
    def initialize(key_error)
      @key = key_error.key
      @keys = key_error.receiver.keys
    end

    def corrections
      @corrections ||= exact_matches.empty? ? SpellChecker.new(dictionary: @keys).correct(@key).map(&:inspect) : exact_matches
    end

    private

    def exact_matches
      @exact_matches ||= @keys.select { |word| @key == word.to_s }.map { |obj| format_object(obj) }
    end

    def format_object(symbol_or_object)
      if symbol_or_object.is_a?(Symbol)
        ":#{symbol_or_object}"
      else
        symbol_or_object.to_s
      end
    end
  end

  class PatternKeyNameChecker
    def initialize(no_matching_pattern_key_error)
      @key = no_matching_pattern_key_error.key
      @keys = no_matching_pattern_key_error.matchee.keys
    end

    def corrections
      @corrections ||= exact_matches.empty? ? SpellChecker.new(dictionary: @keys).correct(@key).map(&:inspect) : exact_matches
    end

    private

    def exact_matches
      @exact_matches ||= @keys.select { |word| @key == word.to_s }.map { |obj| format_object(obj) }
    end

    def format_object(symbol_or_object)
      if symbol_or_object.is_a?(Symbol)
        ":#{symbol_or_object}"
      else
        symbol_or_object.to_s
      end
    end
  end

  class RequirePathChecker
    attr_reader :path

    INITIAL_LOAD_PATH = $LOAD_PATH.dup.freeze
    ENV_SPECIFIC_EXT = ".#{RbConfig::CONFIG["DLEXT"]}"

    private_constant :INITIAL_LOAD_PATH, :ENV_SPECIFIC_EXT

    # The features `require` finds: the files on the load path the program
    # started with, and the libraries metorex carries.
    def self.requireables
      @requireables ||= INITIAL_LOAD_PATH
                          .flat_map { |path| Dir.glob("**/???*{.rb,#{ENV_SPECIFIC_EXT}}", base: path) }
                          .map { |path| path.chomp!(".rb") || path.chomp!(ENV_SPECIFIC_EXT) }
                          .concat(__embedded_library_names__())
    end

    def initialize(exception)
      @path = exception.path
    end

    def corrections
      @corrections ||= begin
        threshold = path.size * 2
        dictionary = self.class.requireables.reject { |str| str.size >= threshold }
        spell_checker = path.include?("/") ? TreeSpellChecker : SpellChecker

        spell_checker.new(dictionary: dictionary).correct(path).uniq
      end
    end
  end

  class Formatter
    def self.message_for(corrections)
      corrections.empty? ? "" : "\nDid you mean?  #{corrections.join("\n               ")}"
    end

    def message_for(corrections)
      warn "The instance method #message_for has been deprecated. Please use the class method " \
           "DidYouMean::Formatter.message_for(...) instead."

      self.class.message_for(corrections)
    end
  end

  PlainFormatter = Formatter
  deprecate_constant :PlainFormatter

  @spell_checkers = Hash.new(NullChecker)
  @formatter = Formatter

  def self.spell_checkers
    @spell_checkers
  end

  # Correct the errors of `error_class` with `spell_checker`, which is
  # handed each error and answers its corrections.
  def self.correct_error(error_class, spell_checker)
    spell_checkers[error_class.to_s] = spell_checker
    error_class.prepend(Correctable) if error_class.is_a?(Class) && !(error_class < Correctable)
  end

  correct_error NameError, NameErrorCheckers
  correct_error KeyError, KeyErrorChecker
  correct_error NoMethodError, MethodNameChecker
  correct_error LoadError, RequirePathChecker
  correct_error NoMatchingPatternKeyError, PatternKeyNameChecker

  def self.formatter
    @formatter
  end

  def self.formatter=(formatter)
    @formatter = formatter
  end
end
