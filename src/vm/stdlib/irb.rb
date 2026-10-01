# The session `binding.irb` opens. It shows the code around the binding,
# then reads Ruby a statement at a time and writes back what each one
# answers. Read from a pipe rather than a terminal, each statement is
# written out before its answer.
module IRB
  KEYWORDS_OPENING_A_LINE = /\A\s*(?:def|class|module|if|unless|while|until|case|begin|for)\b/
  ENDLESS_DEFINITION = /\A\s*def\s+[^\s(]+(?:\([^)]*\))?\s*=\s/

  def self.run_session(held, call_site)
    file, line = held.source_location
    print code_around(file, line)
    interactive = STDIN.tty?
    puts "Switch to inspect mode." unless interactive
    expanded = File.expand_path(file)
    evaluated_as = File.exist?(expanded) ? "#{expanded}(irb)" : expanded
    read = 0
    loop do
      print format("irb(main):%03d> ", read + 1) if interactive
      first = STDIN.gets
      if first.nil?
        puts
        break
      end
      read += 1
      started_on = read
      code = first
      written = [first.chomp]
      while open_depth(code) > 0
        more = STDIN.gets
        break if more.nil?
        read += 1
        written << ("  " * open_depth(code)) + more.strip
        code += more
      end
      puts written unless interactive
      statement = code.strip
      next if statement.empty?
      break if statement == "exit" || statement == "quit"
      begin
        puts held.eval(code, evaluated_as, started_on).inspect
      rescue StandardError, ScriptError => error
        report(error, evaluated_as, started_on, call_site)
      end
    end
    nil
  end

  # The lines around the binding, up to five on either side, with an arrow
  # at the binding's own line.
  def self.code_around(file, line)
    lines = File.read(file).lines
    position = line - 1
    first = [position - 5, 0].max
    last = [position + 5, lines.size - 1].min
    template = " %2s %#{last.to_s.length}d: %s"
    body = (first..last).map do |index|
      format(template, index == position ? "=>" : "", index + 1, lines[index])
    end.join
    "\nFrom: #{file} @ line #{line} :\n\n#{body}\n"
  rescue SystemCallError
    ""
  end

  # How many brackets and blocks the code leaves open, which says whether
  # the statement goes on past the line read so far.
  def self.open_depth(code)
    code.each_line.sum do |line|
      text = line.gsub(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/, '""').sub(/#.*/, "")
      opened = text.count("([{") - text.count(")]}")
      opened += 1 if text.match?(KEYWORDS_OPENING_A_LINE) && !text.match?(ENDLESS_DEFINITION)
      opened += text.scan(/\bdo\b/).size
      opened - text.scan(/\bend\b/).size
    end
  end

  def self.report(error, evaluated_as, started_on, call_site)
    place = error.backtrace&.find { |entry| entry.start_with?(evaluated_as) }
    place ||= "#{evaluated_as}:#{started_on}:in '<main>'"
    puts "#{place}: #{error.message} (#{error.class})"
    unless call_site.empty?
      puts "\tfrom #{call_site.first.sub(/in '[^']*'\z/, "in 'Binding#irb'")}"
      call_site.each { |entry| puts "\tfrom #{entry}" }
    end
  end
end
