$LOAD_PATH.unshift File.join(__dir__, "autoload_fixture")
autoload :Ledger, "ledger"
autoload :Auditing, "auditing"
autoload :Invoice, "invoice"

p defined?(Ledger)
p defined?(::Invoice)

class Ledger
  def balance = total - 25
end
p Ledger.new.balance

module Auditing
  def self.enabled? = level == :full
end
p Auditing.enabled?

p ::Invoice.new.number
