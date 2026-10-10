# frozen_string_literal: true

require "irb/command/debug"

module IRB
  # :stopdoc:

  module Command
    class Backtrace < DebugCommand
      def execute(arg)
        execute_debug_command(pre_cmds: "backtrace #{arg}")
      end
    end
  end

  # :startdoc:
end
