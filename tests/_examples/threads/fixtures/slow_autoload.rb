Holder.recorded << :pre
Thread.current[:loading] = true
sleep 0.1
module Holder
  Loaded = 1
end
Holder.recorded << :post
