# Objects that ask to be told when something changes, and the object that
# tells them.
module Observable
  def add_observer(observer, func = :update)
    @observer_peers = {} if @observer_peers.nil?
    unless observer.respond_to?(func)
      raise NoMethodError, "observer does not respond to '#{func}'"
    end
    @observer_peers[observer] = func
  end

  def delete_observer(observer)
    @observer_peers.delete(observer) unless @observer_peers.nil?
  end

  def delete_observers
    @observer_peers.clear unless @observer_peers.nil?
  end

  def count_observers
    return 0 if @observer_peers.nil?
    @observer_peers.size
  end

  def changed(state = true)
    @observer_state = state
  end

  def changed?
    @observer_state ? true : false
  end

  def notify_observers(*args)
    if @observer_state
      unless @observer_peers.nil?
        @observer_peers.each { |observer, func| observer.send(func, *args) }
      end
      @observer_state = false
    end
  end
end
