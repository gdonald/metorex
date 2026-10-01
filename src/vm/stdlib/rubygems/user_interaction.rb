# How RubyGems talks to whoever runs it: through a UI object that a
# command, a test, or a quiet run can swap out.

require 'rubygems'
require 'rubygems/text'

module Gem
  # Raised to end a gem command with the status it names.
  class SystemExitException < SystemExit
    attr_accessor :exit_code

    def initialize(exit_code)
      @exit_code = exit_code
      super exit_code, "Exiting RubyGems with exit_code #{exit_code}"
    end
  end

  # The UI every interaction goes through until one is swapped in.
  module DefaultUserInteraction
    include Gem::Text

    @ui = nil

    def self.ui
      @ui ||= Gem::ConsoleUI.new
    end

    def self.ui=(new_ui)
      @ui = new_ui
    end

    def self.use_ui(new_ui)
      old_ui = @ui
      @ui = new_ui
      yield
    ensure
      @ui = old_ui
    end

    def ui
      Gem::DefaultUserInteraction.ui
    end

    def ui=(new_ui)
      Gem::DefaultUserInteraction.ui = new_ui
    end

    def use_ui(new_ui, &block)
      Gem::DefaultUserInteraction.use_ui(new_ui, &block)
    end
  end

  # What a command calls to say something, each handed to the UI in force.
  module UserInteraction
    include Gem::DefaultUserInteraction

    def alert(statement, question = nil)
      ui.alert statement, question
    end

    def alert_error(statement, question = nil)
      ui.alert_error statement, question
    end

    def alert_warning(statement, question = nil)
      ui.alert_warning statement, question
    end

    def say(statement = "")
      ui.say statement
    end

    def terminate_interaction(exit_code = 0)
      ui.terminate_interaction exit_code
    end

    # Says the message only when the configuration asks for every detail.
    def verbose(msg = nil)
      say(clean_text(msg || yield)) if Gem.configuration.really_verbose
    end
  end

  # A UI over three streams.
  class StreamUI
    attr_reader :ins, :outs, :errs

    def initialize(in_stream, out_stream, err_stream = $stderr, usetty = true)
      @ins = in_stream
      @outs = out_stream
      @errs = err_stream
      @usetty = usetty
    end

    def tty?
      @usetty && @ins.tty?
    end

    def backtrace(exception)
      return unless Gem.configuration.backtrace

      @errs.puts "\t#{exception.backtrace.join "\n\t"}"
    end

    def say(statement = "")
      @outs.puts statement
    end

    def alert(statement, question = nil)
      @outs.puts "INFO:  #{statement}"
    end

    def alert_warning(statement, question = nil)
      @errs.puts "WARNING:  #{statement}"
    end

    def alert_error(statement, question = nil)
      @errs.puts "ERROR:  #{statement}"
    end

    def terminate_interaction(status = 0)
      close
      raise Gem::SystemExitException, status
    end

    def close
    end
  end

  # The UI a command run from a terminal uses.
  class ConsoleUI < StreamUI
    def initialize
      super $stdin, $stdout, $stderr, true
    end
  end

  # A UI that writes nowhere.
  class SilentUI < StreamUI
    def initialize
      io = NullIO.new
      super io, io, io, false
    end

    def close
    end

    # An IO that takes everything and keeps none of it.
    class NullIO
      def puts(*args)
      end

      def print(*args)
      end

      def flush
      end

      def gets(*args)
      end

      def tty?
        false
      end
    end
  end
end
