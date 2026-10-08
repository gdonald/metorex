# A Hash of Ruby objects kept in a file, read and written in transactions.
# The file holds the table as Marshal writes it.
class PStore
  # Raised for anything done outside a transaction, for a change made inside
  # a read-only one, and for a key `fetch` cannot find.
  class Error < StandardError
  end

  # A data file is written to a new name and moved over the old one when this
  # is true, so a crash part-way leaves the old file whole.
  attr_accessor :ultra_safe

  attr_reader :path

  def initialize(file, thread_safe = false)
    @filename = file
    @path = file
    @abort = false
    @ultra_safe = false
    @thread_safe = thread_safe
    @lock = Thread::Mutex.new
    @table = nil
    @in_transaction = false
    @read_only = false
  end

  def [](key)
    in_transaction
    @table[key]
  end

  def []=(key, value)
    in_transaction_writable
    @table[key] = value
  end

  def fetch(key, default = NOT_GIVEN)
    in_transaction
    return @table[key] if @table.key?(key)
    raise PStore::Error, "undefined key '#{key}'" if default.equal?(NOT_GIVEN)
    default
  end

  def delete(key)
    in_transaction_writable
    @table.delete(key)
  end

  def roots
    in_transaction
    @table.keys
  end

  alias keys roots

  def root?(key)
    in_transaction
    @table.key?(key)
  end

  alias key? root?

  # Leave the transaction now, writing what it changed.
  def commit
    in_transaction
    @abort = false
    throw :pstore_abort_transaction
  end

  # Leave the transaction now, keeping none of what it changed.
  def abort
    in_transaction
    @abort = true
    throw :pstore_abort_transaction
  end

  # Run the block with the table read from the file, and write the table
  # back afterwards unless the transaction is read-only or was aborted. An
  # exception the block raises discards its changes too.
  def transaction(read_only = false)
    raise PStore::Error, "nested transaction" if @in_transaction

    @lock.lock if @thread_safe
    begin
      @in_transaction = true
      @read_only = read_only
      @abort = false
      @table = load_table
      answer = nil
      catch(:pstore_abort_transaction) do
        answer = yield(self)
      end
      save_table unless read_only || @abort
      answer
    ensure
      @table = nil
      @in_transaction = false
      @lock.unlock if @thread_safe
    end
  end

  NOT_GIVEN = Object.new
  private_constant :NOT_GIVEN

  private

  def in_transaction
    raise PStore::Error, "not in transaction" unless @in_transaction
  end

  def in_transaction_writable
    in_transaction
    raise PStore::Error, "in read-only transaction" if @read_only
  end

  def load_table
    return {} unless File.exist?(@filename)

    data = File.binread(@filename)
    data.empty? ? {} : Marshal.load(data)
  end

  def save_table
    data = Marshal.dump(@table)
    if @ultra_safe
      temporary = "#{@filename}.tmp#{Process.pid}"
      File.binwrite(temporary, data)
      File.rename(temporary, @filename)
    else
      File.binwrite(@filename, data)
    end
  end
end
