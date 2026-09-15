require 'socket'
require 'openssl'
require 'timeout'
require 'monitor'
require 'net/http'

module Net
  class FTPError < StandardError
  end

  class FTPReplyError < FTPError
  end

  class FTPTempError < FTPError
  end

  class FTPPermError < FTPError
  end

  class FTPProtoError < FTPError
  end

  class FTPConnectionError < FTPError
  end

  # The socket an FTP session holds before it has connected to anything,
  # which answers every question the way a closed one does.
  class FTPNullSocket
    def closed?
      true
    end

    def close
      nil
    end

    def read_timeout
      nil
    end

    def read_timeout= seconds
      seconds
    end

    def method_missing name, *args
      raise FTPConnectionError, 'not connected'
    end
  end

  class FTP < Protocol
    FTP_PORT = 21
    CRLF = "\r\n"
    DEFAULT_BLOCKSIZE = 4096
    VERSION = '0.3.8'

    @@default_passive = true

    def self.default_passive
      @@default_passive
    end

    def self.default_passive= value
      @@default_passive = value
    end

    def self.open host, *args
      return new(host, *args) unless block_given?
      ftp = new host, *args
      begin
        yield ftp
      ensure
        ftp.close
      end
    end

    def initialize host = nil, user_or_options = {}, passwd = nil, acct = nil
      super()
      begin
        options = user_or_options.to_hash
      rescue NoMethodError
        options = {}
        options[:username] = user_or_options
        options[:password] = passwd
        options[:account] = acct
      end
      @host = nil
      if options[:ssl]
        ssl_params = options[:ssl] == true ? {} : options[:ssl]
        @ssl_context = OpenSSL::SSL::SSLContext.new
        @ssl_context.set_params ssl_params
        @ssl_session = nil
        @private_data_connection = options[:private_data_connection]
        @private_data_connection = true if @private_data_connection.nil?
      else
        @ssl_context = nil
        if options[:private_data_connection]
          raise ArgumentError, 'private_data_connection can be set to true only when ssl is enabled'
        end
        @private_data_connection = false
      end
      @binary = true
      @passive = options[:passive].nil? ? @@default_passive : options[:passive]
      @debug_mode = options[:debug_mode].nil? ? false : options[:debug_mode]
      @resume = false
      @bare_sock = FTPNullSocket.new
      @sock = @bare_sock
      @logged_in = false
      @open_timeout = options[:open_timeout]
      @ssl_handshake_timeout = options[:ssl_handshake_timeout]
      @read_timeout = options[:read_timeout] || 60
      @use_pasv_ip = options[:use_pasv_ip]
      @last_response = nil
      @last_response_code = nil
      @welcome = nil
      @mtime_zone = nil
      return unless host
      connect host, options[:port] || FTP_PORT
      login options[:username], options[:password], options[:account] if options[:username]
    end
    private :initialize

    attr_accessor :binary
    attr_accessor :passive
    attr_accessor :debug_mode
    attr_accessor :resume
    attr_accessor :open_timeout
    attr_accessor :ssl_handshake_timeout
    attr_accessor :use_pasv_ip
    attr_reader :welcome
    attr_reader :last_response
    attr_reader :last_response_code

    # The name the response code went by before Ruby 1.9 named it in full.
    def lastresp
      @last_response_code
    end
    attr_reader :read_timeout

    def read_timeout= seconds
      @sock.read_timeout = seconds if @sock && !@sock.closed?
      @read_timeout = seconds
    end

    def return_code
      warn 'warning: Net::FTP#return_code is obsolete and do nothing'
      "\n"
    end

    def return_code= code
      warn 'warning: Net::FTP#return_code= is obsolete and do nothing'
      code
    end

    def closed?
      @sock.nil? || @sock.closed?
    end

    def close
      return nil if @sock.nil? || @sock.closed?
      begin
        @sock.read_timeout = 3
        @sock.close
      ensure
        @sock = nil
        @bare_sock = nil
      end
      nil
    end

    def set_socket sock, get_greeting = true
      @sock = sock
      voidresp if get_greeting
      sock
    end

    def connect host, port = FTP_PORT
      @host = host
      @bare_sock = TCPSocket.new host, port
      @sock = @bare_sock
      @sock.read_timeout = @read_timeout if @sock.respond_to? :read_timeout=
      voidresp
      nil
    end

    def login user = 'anonymous', passwd = nil, acct = nil
      passwd = "anonymous@" if user == 'anonymous' && passwd.nil?
      reply = sendcmd "USER #{user}"
      if reply.start_with? '3'
        raise FTPReplyError, reply if passwd.nil?
        reply = sendcmd "PASS #{passwd}"
      end
      if reply.start_with? '3'
        raise FTPReplyError, reply if acct.nil?
        reply = sendcmd "ACCT #{acct}"
      end
      raise FTPReplyError, reply unless reply.start_with? '2'
      @logged_in = true
      # The reply that logged the program in is the message the server is
      # remembered by, which is what `welcome` answers.
      @welcome = reply
      voidcmd(@binary ? 'TYPE I' : 'TYPE A')
      reply
    end

    def putline line
      @sock.write "#{line}#{CRLF}"
      $stderr.puts "put: #{line}" if @debug_mode
      line
    end
    private :putline

    def getline
      line = @sock.gets
      raise EOFError, 'end of file reached' if line.nil?
      line = line.sub(/\r?\n\z/, '')
      $stderr.puts "get: #{line}" if @debug_mode
      line
    end
    private :getline

    def getmultiline
      lines = [getline]
      if /\A(\d\d\d)-/.match lines[0]
        code = Regexp.last_match(1)
        loop do
          line = getline
          lines = lines + [line]
          break if line.start_with?("#{code} ") || line == code
        end
      end
      "#{lines.join "\n"}\n"
    end
    private :getmultiline

    def getresp
      @last_response = getmultiline
      @last_response_code = @last_response[0, 3]
      case @last_response_code[0, 1]
      when '1', '2', '3'
        @last_response
      when '4'
        raise FTPTempError, @last_response
      when '5'
        raise FTPPermError, @last_response
      else
        raise FTPProtoError, @last_response
      end
    end

    # A command with nothing to answer checks the reply and reports nothing,
    # which is what makes `noop` and `quit` answer nil.
    def voidresp
      reply = getresp
      raise FTPReplyError, reply unless reply.start_with? '2'
      nil
    end

    def sendcmd cmd
      putline cmd
      getresp
    end

    def voidcmd cmd
      putline cmd
      voidresp
    end

    def noop
      voidcmd 'NOOP'
    end

    def site arg
      voidcmd "SITE #{arg}"
    end

    # `abort` reads its answer without recording it, so the last response a
    # program asked about is still the one before it.
    def abort
      putline 'ABOR'
      reply = getmultiline
      unless %w[426 226 225].include? reply[0, 3]
        raise FTPProtoError, reply
      end
      reply
    end

    def acct account
      voidcmd "ACCT #{account}"
    end

    def status pathname = nil
      pathname.nil? ? sendcmd('STAT') : sendcmd("STAT #{pathname}")
    end

    def system
      sendcmd('SYST').sub(/\A\d\d\d /, '').chomp
    end

    def pwd
      reply = sendcmd 'PWD'
      parse257 reply
    end

    def getdir
      pwd
    end

    def chdir dirname
      if dirname == '..'
        begin
          return voidcmd('CDUP')
        rescue FTPPermError => problem
          raise problem unless problem.message.start_with? '500'
        end
      end
      voidcmd "CWD #{dirname}"
    end

    def mkdir dirname
      parse257 sendcmd("MKD #{dirname}")
    end

    def rmdir dirname
      voidcmd "RMD #{dirname}"
    end

    def delete filename
      reply = sendcmd "DELE #{filename}"
      return reply if reply.start_with? '250'
      raise FTPReplyError, reply
    end

    def rename fromname, toname
      reply = sendcmd "RNFR #{fromname}"
      raise FTPReplyError, reply unless reply.start_with? '3'
      voidcmd "RNTO #{toname}"
    end

    def size filename
      voidcmd 'TYPE I'
      parse213(sendcmd("SIZE #{filename}")).to_i
    end

    def mdtm filename
      parse213 sendcmd("MDTM #{filename}")
    end

    def help arg = nil
      arg.nil? ? sendcmd('HELP') : sendcmd("HELP #{arg}")
    end

    def quit
      voidcmd 'QUIT'
    end

    # Open the connection the data travels over. In passive mode the server
    # names a port to reach it on, and otherwise the program listens on one
    # of its own and tells the server where to find it.
    def transfercmd cmd, rest_offset = nil
      if @passive
        host, port = parse227 sendcmd('PASV')
        restart_at rest_offset if rest_offset
        opened = TCPSocket.new host, port
        begin
          reply = sendcmd cmd
          raise FTPReplyError, reply unless reply.start_with?('1') || reply.start_with?('2')
        rescue StandardError
          opened.close
          raise
        end
        return opened
      end
      listener = TCPServer.new @bare_sock.addr[3], 0
      begin
        named = listener.addr
        sendport named[3], named[1]
        restart_at rest_offset if rest_offset
        reply = sendcmd cmd
        raise FTPReplyError, reply unless reply.start_with?('1') || reply.start_with?('2')
        listener.accept
      ensure
        listener.close
      end
    end

    # Tell the server where in the file the transfer picks up. It answers
    # that it is waiting for the command itself, which is a 3xx reply.
    def restart_at offset
      reply = sendcmd "REST #{offset}"
      raise FTPReplyError, reply unless reply.start_with? '3'
      reply
    end
    private :restart_at

    def sendport host, port
      voidcmd format('PORT %s,%d,%d', host.tr('.', ','), port >> 8, port & 0xff)
    end
    private :sendport

    def parse227 reply
      raise FTPReplyError, reply unless reply.start_with? '227'
      matched = /\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)/.match reply
      raise FTPProtoError, reply if matched.nil?
      numbers = matched[1, 6].map { |part| part.to_i }
      [numbers[0, 4].join('.'), numbers[4] * 256 + numbers[5]]
    end
    private :parse227

    # Read what a command answers a line at a time, handing each line to the
    # block. Without a block the lines are collected and answered.
    def retrlines cmd
      collected = block_given? ? nil : []
      opened = transfercmd cmd
      begin
        while (line = opened.gets)
          line = line.sub(/\r?\n\z/, '')
          if block_given?
            yield line
          else
            collected << line
          end
        end
      ensure
        opened.close
      end
      voidresp
      collected
    end

    # Read what a command answers in pieces of `blocksize` bytes.
    def retrbinary cmd, blocksize = DEFAULT_BLOCKSIZE, rest_offset = nil
      opened = transfercmd cmd, rest_offset
      begin
        while (held = opened.read(blocksize))
          break if held.empty?
          yield held
        end
      ensure
        opened.close
      end
      voidresp
    end

    # Write what a stream holds a line at a time, each ending the way the
    # protocol asks for.
    def storlines cmd, file
      opened = transfercmd cmd
      begin
        while (line = file.gets)
          line = line.chomp + CRLF unless line.end_with? CRLF
          opened.write line
          yield line if block_given?
        end
      ensure
        opened.close
      end
      voidresp
    end

    def storbinary cmd, file, blocksize = DEFAULT_BLOCKSIZE, rest_offset = nil
      # Picking up where a transfer left off starts reading the local file at
      # the same place the remote one ends.
      file.seek rest_offset if rest_offset
      opened = transfercmd cmd, rest_offset
      begin
        while (held = file.read(blocksize))
          break if held.empty?
          opened.write held
          yield held if block_given?
        end
      ensure
        opened.close
      end
      voidresp
    end

    def list(*args, &block)
      cmd = (['LIST'] + args).join ' '
      return retrlines(cmd, &block) if block
      retrlines cmd
    end
    alias ls list
    alias dir list

    def nlst dir = nil
      cmd = dir.nil? ? 'NLST' : "NLST #{dir}"
      retrlines cmd
    end

    def getbinaryfile remotefile, localfile = File.basename(remotefile),
                      blocksize = DEFAULT_BLOCKSIZE
      offset = @resume && localfile && File.exist?(localfile) ? File.size(localfile) : nil
      writing = localfile.nil? ? nil : File.open(localfile, offset ? 'ab' : 'wb')
      # With nowhere to write it, the whole of what was read is the answer.
      collected = writing.nil? && !block_given? ? +'' : nil
      begin
        retrbinary("RETR #{remotefile}", blocksize, offset) do |held|
          writing.write held unless writing.nil?
          collected << held unless collected.nil?
          yield held if block_given?
        end
      ensure
        writing.close unless writing.nil?
      end
      collected
    end

    def gettextfile remotefile, localfile = File.basename(remotefile)
      writing = localfile.nil? ? nil : File.open(localfile, 'w')
      collected = writing.nil? && !block_given? ? +'' : nil
      begin
        retrlines("RETR #{remotefile}") do |line|
          writing.puts line unless writing.nil?
          collected << line << "\n" unless collected.nil?
          yield line if block_given?
        end
      ensure
        writing.close unless writing.nil?
      end
      collected
    end

    def get remotefile, localfile = File.basename(remotefile),
            blocksize = DEFAULT_BLOCKSIZE, &block
      if @binary
        getbinaryfile remotefile, localfile, blocksize, &block
      else
        gettextfile remotefile, localfile, &block
      end
    end

    def putbinaryfile localfile, remotefile = File.basename(localfile),
                      blocksize = DEFAULT_BLOCKSIZE, &block
      # Picking up where a file left off appends to what is already there,
      # which is a different command from writing it afresh.
      offset = nil
      if @resume
        begin
          offset = size remotefile
        rescue FTPPermError
          offset = nil
        end
      end
      File.open(localfile, 'rb') do |reading|
        if offset
          storbinary "APPE #{remotefile}", reading, blocksize, offset, &block
        else
          storbinary "STOR #{remotefile}", reading, blocksize, nil, &block
        end
      end
      nil
    end

    def puttextfile localfile, remotefile = File.basename(localfile), &block
      File.open(localfile) do |reading|
        storlines "STOR #{remotefile}", reading, &block
      end
      @last_response
    end

    def put localfile, remotefile = File.basename(localfile),
            blocksize = DEFAULT_BLOCKSIZE, &block
      if @binary
        putbinaryfile localfile, remotefile, blocksize, &block
      else
        puttextfile localfile, remotefile, &block
      end
    end

    # When a file was last written, read from what MDTM answers. The time is
    # UTC unless the local zone was asked for.
    def mtime filename, local = false
      spelled = mdtm(filename).strip
      matched = /\A(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/.match spelled
      raise FTPProtoError, "invalid time-val: #{spelled}" if matched.nil?
      parts = matched[1, 6].map { |part| part.to_i }
      if local
        Time.local parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
      else
        Time.utc parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]
      end
    end

    def parse257 reply
      matched = /\A257 "(.*)"/.match reply
      raise FTPProtoError, reply if matched.nil?
      matched[1].gsub '""', '"'
    end
    private :parse257

    def parse213 reply
      reply.sub(/\A\d\d\d /, '').strip
    end
    private :parse213
  end
end
