#include "ruby.h"

static VALUE body(RB_BLOCK_CALL_FUNC_ARGLIST(first, data)) {
  VALUE values[4] = {first, data, INT2FIX(argc), rb_ary_new_from_values(argc, argv)};
  return rb_ary_new_from_values(4, values);
}

static VALUE fiber_new(VALUE self, VALUE data) { return rb_fiber_new(body, data); }
static VALUE current(VALUE self) { return rb_fiber_current(); }
static VALUE alive(VALUE self, VALUE fiber) { return rb_fiber_alive_p(fiber); }
static VALUE resume(VALUE self, VALUE fiber, VALUE values) {
  return rb_fiber_resume(fiber, (int)RARRAY_LEN(values), RARRAY_PTR(values));
}
static VALUE yield(VALUE self, VALUE values) {
  return rb_fiber_yield((int)RARRAY_LEN(values), RARRAY_PTR(values));
}
static VALUE transfer(VALUE self, VALUE fiber, VALUE values) {
  return rb_fiber_transfer(fiber, (int)RARRAY_LEN(values), RARRAY_PTR(values));
}
static VALUE raise_in(VALUE self, VALUE fiber, VALUE values) {
  return rb_fiber_raise(fiber, (int)RARRAY_LEN(values), RARRAY_PTR(values));
}
static VALUE entry(VALUE self, VALUE array, VALUE offset) {
  return rb_ary_entry(array, FIX2LONG(offset));
}

void Init_c_fibers(void) {
  VALUE cls = rb_define_class("CFibers", rb_cObject);
  rb_define_method(cls, "fiber_new", fiber_new, 1);
  rb_define_method(cls, "current", current, 0);
  rb_define_method(cls, "alive", alive, 1);
  rb_define_method(cls, "resume", resume, 2);
  rb_define_method(cls, "yield", yield, 1);
  rb_define_method(cls, "transfer", transfer, 2);
  rb_define_method(cls, "raise_in", raise_in, 2);
  rb_define_method(cls, "entry", entry, 2);
}
