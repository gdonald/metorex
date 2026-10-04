# A Ractor receives what is sent to a port it made. Its default port takes
# what `Ractor#send` sends, a value is copied on the way unless it is
# shareable, and one sent with `move: true` leaves nothing usable behind.
# `Ractor.select` answers the first port or Ractor with something to hand over.

Warning[:experimental] = false
p Ractor.current.default_port
port = Ractor::Port.new
p port
port << 1; port.send 2
p port.receive, port.receive
r = Ractor.new(port) { |pt| pt << Ractor.receive * 10; :ok }
r.send 4
p port.receive, r.value
p r.default_port.class, Ractor.current.default_port.class
reader = Ractor.new(port) { |pt| begin; pt.receive; rescue => e; [e.class, e.message]; end }
p reader.value
p Ractor::Port.new
p reader.default_port
port.close; p port.closed?
begin; port << 1; rescue => e; p [e.class, e.message]; end
begin; port.receive; rescue => e; p [e.class, e.message]; end
a = Ractor::Port.new; b = Ractor::Port.new
a << :x
p Ractor.select(a, b)
r2 = Ractor.new { :value_done }
p Ractor.select(r2).map { |held| held.is_a?(Ractor) ? held.class : held }
s = +"moved"
q = Ractor::Port.new
q.send s, move: true
v = q.receive
p v, v.equal?(s)
begin; p s; rescue => e; p [e.class, e.message]; end
begin; s.inspect; rescue => e; p [e.class, e.message]; end
begin; s.class; rescue => e; p [e.class, e.message]; end
p Ractor::MovedObject.superclass
x = [1]; q << x; y = q.receive; p y.equal?(x), y == x
p Ractor::Port.instance_methods(false).sort
w = Ractor::Port.new
begin; p Ractor.select(w, timeout: 0.01); rescue => e; p [e.class, e.message]; end
begin; Ractor.select; rescue => e; p [e.class, e.message]; end
