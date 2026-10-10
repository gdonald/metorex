# frozen_string_literal: true

require "irb/command/debug"

module IRB
  # :stopdoc:

  module Command
    class Finish < DebugCommand
      def execute(arg)
        execute_debug_command(do_cmds: "finish #{arg}")
      end
    end
  end

  # :startdoc:
end
