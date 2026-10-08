# The commands `ruby -run -e` runs: file operations a Makefile can call the
# same way on every platform, an HTTP server, a Makefile generator, and the
# help for each. Each reads its options with OptionParser and works through
# FileUtils.
require "fileutils"
require "optparse"

module FileUtils
  @fileutils_output = $stdout
end

# Read the options `options` names, one letter each with a colon after one
# that takes an argument, and the long ones `long_options` names in camel
# case, then hand the arguments left and the options read to the block. An
# argument with a glob character in it stands for the paths it matches.
def setup(options = "", *long_options)
  caller_label = caller_locations(1, 1)[0].label
  read = {}
  arguments = []
  OptionParser.new do |parser|
    options.scan(/.:?/) do |letter|
      name = letter.delete(":").intern
      parser.on("-" + letter.tr(":", " ")) { |value| read[name] = value }
    end
    long_options.each do |long|
      name, argument_name = long.split(/(?=[\s=])/, 2)
      name = name.delete_prefix("--")
      spelled = "--#{name.gsub(/([A-Z]+|[a-z])([A-Z])/, '\\1-\\2').downcase}#{argument_name}"
      puts "#{name}=>#{spelled}" if $DEBUG
      name = name.intern
      parser.on(spelled) { |value| read[name] = value }
    end
    parser.on("-v") { read[:verbose] = true }
    parser.on("--help") do
      UN.help([caller_label])
      exit
    end
    parser.order!(ARGV) do |argument|
      if /[*?\[{]/ =~ argument
        arguments.concat(Dir[argument])
      else
        arguments << argument
      end
    end
  end
  yield arguments, read
end

def cp
  setup("prl") do |arguments, options|
    command = "cp"
    command += "_r" if options.delete :r
    command = "cp_lr" if options.delete :l
    options[:preserve] = true if options.delete :p
    destination = arguments.pop
    arguments = arguments[0] if arguments.size == 1
    FileUtils.__send__ command, arguments, destination, **options
  end
end

def ln
  setup("sf") do |arguments, options|
    command = "ln"
    command += "_s" if options.delete :s
    options[:force] = true if options.delete :f
    destination = arguments.pop
    arguments = arguments[0] if arguments.size == 1
    FileUtils.__send__ command, arguments, destination, **options
  end
end

def mv
  setup do |arguments, options|
    destination = arguments.pop
    arguments = arguments[0] if arguments.size == 1
    FileUtils.mv arguments, destination, **options
  end
end

def rm
  setup("fr") do |arguments, options|
    command = "rm"
    command += "_r" if options.delete :r
    options[:force] = true if options.delete :f
    FileUtils.__send__ command, arguments, **options
  end
end

def mkdir
  setup("p") do |arguments, options|
    command = "mkdir"
    command += "_p" if options.delete :p
    FileUtils.__send__ command, arguments, **options
  end
end

def rmdir
  setup("p") do |arguments, options|
    options[:parents] = true if options.delete :p
    FileUtils.rmdir arguments, **options
  end
end

def install
  setup("pm:o:g:") do |arguments, options|
    (mode = options.delete :m) and options[:mode] = /\A\d/ =~ mode ? mode.oct : mode
    options[:preserve] = true if options.delete :p
    (owner = options.delete :o) and options[:owner] = owner
    (group = options.delete :g) and options[:group] = group
    destination = arguments.pop
    arguments = arguments[0] if arguments.size == 1
    FileUtils.install arguments, destination, **options
  end
end

def chmod
  setup do |arguments, options|
    mode = arguments.shift
    mode = /\A\d/ =~ mode ? mode.oct : mode
    FileUtils.chmod mode, arguments, **options
  end
end

def touch
  setup do |arguments, options|
    FileUtils.touch arguments, **options
  end
end

# Wait until each file can be opened for writing, trying again every `-w`
# seconds, at most `-n` times.
def wait_writable
  setup("n:w:v") do |arguments, options|
    verbose = options[:verbose]
    tries = options[:n] and tries = Integer(tries)
    pause = (pause = options[:w]) ? Float(pause) : 0.2
    arguments.each do |file|
      begin
        File.open(file, "r+b") { }
      rescue Errno::ENOENT
        break
      rescue Errno::EACCES => error
        raise if tries and (tries -= 1) <= 0
        if verbose
          puts error
          STDOUT.flush
        end
        sleep pause
        retry
      end
    end
  end
end

def mkmf
  setup("d:h:l:f:v:t:m:c:", "vendor") do |arguments, options|
    require "mkmf"
    (found = options[:d]) and found.split(/:/).each { |named| dir_config(*named.split(/,/)) }
    (found = options[:h]) and found.split(/:/).each { |named| have_header(*named.split(/,/)) }
    (found = options[:l]) and found.split(/:/).each { |named| have_library(*named.split(/,/)) }
    (found = options[:f]) and found.split(/:/).each { |named| have_func(*named.split(/,/)) }
    (found = options[:v]) and found.split(/:/).each { |named| have_var(*named.split(/,/)) }
    (found = options[:t]) and found.split(/:/).each { |named| have_type(*named.split(/,/)) }
    (found = options[:m]) and found.split(/:/).each { |named| have_macro(*named.split(/,/)) }
    (found = options[:c]) and found.split(/:/).each { |named| have_const(*named.split(/,/)) }
    $configure_args["--vendor"] = true if options[:vendor]
    create_makefile(*arguments)
  end
end

# Serve a directory over HTTP with WEBrick, which is a gem of its own.
def httpd
  setup("", "BindAddress=ADDR", "Port=PORT", "MaxClients=NUM", "TempDir=DIR",
        "DoNotReverseLookup", "RequestTimeout=SECOND", "HTTPVersion=VERSION",
        "ServerName=NAME", "ServerSoftware=NAME",
        "SSLCertificate=CERT", "SSLPrivateKey=KEY") do |arguments, options|
    begin
      require "webrick"
    rescue LoadError
      abort "webrick is not found. You may need to `gem install webrick` to install webrick."
    end
    (given = options[:RequestTimeout]) and options[:RequestTimeout] = given.to_i
    [:Port, :MaxClients].each do |name|
      begin
        (given = options[name]) and (options[name] = Integer(given))
      rescue StandardError
        nil
      end
    end
    if (certificate = options[:SSLCertificate])
      key = options[:SSLPrivateKey] or raise "--ssl-private-key option must also be given"
      require "webrick/https"
      options[:SSLEnable] = true
      options[:SSLCertificate] = OpenSSL::X509::Certificate.new(File.read(certificate))
      options[:SSLPrivateKey] = OpenSSL::PKey.read(File.read(key))
      options[:Port] ||= 8443
    end
    options[:Port] ||= 8080
    options[:DocumentRoot] = arguments.shift || "."
    server = nil
    options[:StartCallback] = proc do
      logger = server.logger
      logger.info("To access this server, open this URL in a browser:")
      server.listeners.each do |listener|
        if options[:SSLEnable]
          address = listener.addr
          address[3] = "127.0.0.1" if address[3] == "0.0.0.0"
          address[3] = "::1" if address[3] == "::"
          logger.info("    https://#{Addrinfo.new(address).inspect_sockaddr}")
        else
          logger.info("    http://#{listener.connect_address.inspect_sockaddr}")
        end
      end
    end
    server = WEBrick::HTTPServer.new(options)
    stop = proc { server.shutdown }
    signals = %w[TERM QUIT]
    signals.concat(%w[HUP INT]) if STDIN.tty?
    signals &= Signal.list.keys
    signals.each { |signal| Signal.trap(signal, stop) }
    server.start
  end
end

def colorize
  begin
    require "irb/color"
  rescue LoadError
    raise "colorize requires irb 1.1.0 or later"
  end
  setup do |arguments, |
    if arguments.empty?
      puts IRB::Color.colorize_code STDIN.read
      return
    end
    arguments.each { |file| puts IRB::Color.colorize_code File.read(file) }
  end
end

def help
  setup do |arguments, |
    UN.help(arguments)
  end
end

module UN
  VERSION = "0.3.0"

  # What `help` prints for each command, in the order MRI's library lists
  # them.
  HELP = {
    "cp" => "Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY\n\n  ruby -run -e cp -- [OPTION] SOURCE DEST\n\n  -p          preserve file attributes if possible\n  -r          copy recursively\n  -l          make hard link instead of copying (implies -r)\n  -v          verbose\n\n\n",
    "ln" => "Create a link to the specified TARGET with LINK_NAME.\n\n  ruby -run -e ln -- [OPTION] TARGET LINK_NAME\n\n  -s          make symbolic links instead of hard links\n  -f          remove existing destination files\n  -v          verbose\n\n\n",
    "mv" => "Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.\n\n  ruby -run -e mv -- [OPTION] SOURCE DEST\n\n  -v          verbose\n\n\n",
    "rm" => "Remove the FILE\n\n  ruby -run -e rm -- [OPTION] FILE\n\n  -f          ignore nonexistent files\n  -r          remove the contents of directories recursively\n  -v          verbose\n\n\n",
    "mkdir" => "Create the DIR, if they do not already exist.\n\n  ruby -run -e mkdir -- [OPTION] DIR\n\n  -p          no error if existing, make parent directories as needed\n  -v          verbose\n\n\n",
    "rmdir" => "Remove the DIR.\n\n  ruby -run -e rmdir -- [OPTION] DIR\n\n  -p          remove DIRECTORY and its ancestors.\n  -v          verbose\n\n\n",
    "install" => "Copy SOURCE to DEST.\n\n  ruby -run -e install -- [OPTION] SOURCE DEST\n\n  -p          apply access/modification times of SOURCE files to\n              corresponding destination files\n  -m          set permission mode (as in chmod), instead of 0755\n  -o          set owner user id, instead of the current owner\n  -g          set owner group id, instead of the current group\n  -v          verbose\n\n\n",
    "chmod" => "Change the mode of each FILE to OCTAL-MODE.\n\n  ruby -run -e chmod -- [OPTION] OCTAL-MODE FILE\n\n  -v          verbose\n\n\n",
    "touch" => "Update the access and modification times of each FILE to the current time.\n\n  ruby -run -e touch -- [OPTION] FILE\n\n  -v          verbose\n\n\n",
    "wait_writable" => "Wait until the file becomes writable.\n\n  ruby -run -e wait_writable -- [OPTION] FILE\n\n  -n RETRY    count to retry\n  -w SEC      each wait time in seconds\n  -v          verbose\n\n\n",
    "mkmf" => "Create makefile using mkmf.\n\n  ruby -run -e mkmf -- [OPTION] EXTNAME [OPTION]\n\n  -d ARGS     run dir_config\n  -h ARGS     run have_header\n  -l ARGS     run have_library\n  -f ARGS     run have_func\n  -v ARGS     run have_var\n  -t ARGS     run have_type\n  -m ARGS     run have_macro\n  -c ARGS     run have_const\n  --vendor    install to vendor_ruby\n\n\n",
    "httpd" => "Run WEBrick HTTP server.\n\n  ruby -run -e httpd -- [OPTION] [DocumentRoot]\n\n  --bind-address=ADDR         address to bind\n  --port=NUM                  listening port number\n  --max-clients=MAX           max number of simultaneous clients\n  --temp-dir=DIR              temporary directory\n  --do-not-reverse-lookup     disable reverse lookup\n  --request-timeout=SECOND    request timeout in seconds\n  --http-version=VERSION      HTTP version\n  --server-name=NAME          name of the server host\n  --server-software=NAME      name and version of the server\n  --ssl-certificate=CERT      The SSL certificate file for the server\n  --ssl-private-key=KEY       The SSL private key file for the server certificate\n  -v                          verbose\n\n\n",
    "colorize" => "Colorize ruby code.\n\n  ruby -run -e colorize -- [FILE]\n\n\n",
    "help" => "Display help message.\n\n  ruby -run -e help [COMMAND]\n\n\n",
  }.freeze

  module_function

  # The help for each command named, or for every command when none is.
  def help(commands, output: $stdout)
    if commands.empty?
      HELP.each_value { |text| output << text }
      return
    end
    commands.each { |command| output << HELP[command] }
  end
end
