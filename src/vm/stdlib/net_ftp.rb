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
      @welcome = voidresp
      nil
    end

    def login user = 'anonymous', passwd = nil, acct = nil
      passwd = "anonymous@" if user == 'anonymous' && passwd.nil?
      reply = sendcmd "USER #{user}"
      reply = sendcmd "PASS #{passwd}" if reply.start_with? '3'
      reply = sendcmd "ACCT #{acct}" if reply.start_with? '3'
      raise FTPReplyError, reply unless reply.start_with? '2'
      @logged_in = true
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

    def voidresp
      reply = getresp
      raise FTPReplyError, reply unless reply.start_with? '2'
      reply
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

    def abort
      putline 'ABOR'
      getresp
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
      voidcmd "RNFR #{fromname}"
      voidcmd "RNTO #{toname}"
    end

    def size filename
      voidcmd 'TYPE I'
      parse213 sendcmd("SIZE #{filename}")
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
