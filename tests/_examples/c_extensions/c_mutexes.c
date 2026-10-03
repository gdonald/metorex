#include "ruby.h"

static VALUE make(VALUE self) { return rb_mutex_new(); }
static VALUE locked(VALUE self, VALUE mutex) { return rb_mutex_locked_p(mutex); }
static VALUE try_lock(VALUE self, VALUE mutex) { return rb_mutex_trylock(mutex); }
static VALUE lock(VALUE self, VALUE mutex) { return rb_mutex_lock(mutex); }
static VALUE unlock(VALUE self, VALUE mutex) { return rb_mutex_unlock(mutex); }
static VALUE sleep_on(VALUE self, VALUE mutex, VALUE timeout) { return rb_mutex_sleep(mutex, timeout); }
static VALUE run(VALUE callable) { return rb_funcall(callable, rb_intern("call"), 0); }
static VALUE synchronize(VALUE self, VALUE mutex, VALUE callable) {
  return rb_mutex_synchronize(mutex, run, callable);
}

void Init_c_mutexes(void) {
  VALUE cls = rb_define_class("CMutexes", rb_cObject);
  rb_define_method(cls, "make", make, 0);
  rb_define_method(cls, "locked", locked, 1);
  rb_define_method(cls, "try_lock", try_lock, 1);
  rb_define_method(cls, "lock", lock, 1);
  rb_define_method(cls, "unlock", unlock, 1);
  rb_define_method(cls, "sleep_on", sleep_on, 2);
  rb_define_method(cls, "synchronize", synchronize, 2);
}
