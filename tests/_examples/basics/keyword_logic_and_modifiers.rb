rate = if 2 > 1
         nil or
           7
       else
         0
       end
p rate

if (fee = rate - 2) and fee > 3
  p fee
end
total = (tax = 4 and tax + 1)
p [total, tax]
fallback = (missing = nil or 9)
p [fallback, missing]

class Shipping; end if true
class Freight; end if false
module Billing; end unless false
def audited; :audited; end if true
def skipped; end if false
p [defined?(Shipping), defined?(Freight), defined?(Billing), defined?(skipped)]
p audited

notice = <<'.,.,'
maintenance window moves to 02:00
.,.,
p notice
p <<'-END', 2
first line
-END
