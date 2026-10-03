#include "ruby.h"

static VALUE info(VALUE self) { return rb_errinfo(); }
static VALUE set_info(VALUE self, VALUE exception) {
  rb_set_errinfo(exception);
  return rb_errinfo();
}
static VALUE from_bytes(VALUE self, VALUE text) {
  return rb_exc_new(rb_eArgError, RSTRING_PTR(text), RSTRING_LEN(text));
}
static VALUE from_c_string(VALUE self, VALUE text) { return rb_exc_new2(rb_eIOError, RSTRING_PTR(text)); }
static VALUE from_string(VALUE self, VALUE text) { return rb_exc_new3(rb_eTypeError, text); }
static VALUE raise_it(VALUE self, VALUE exception) { rb_exc_raise(exception); }
static VALUE frozen(VALUE self, VALUE object) { rb_error_frozen_object(object); }
static VALUE system_error(VALUE self, VALUE number, VALUE message) {
  return rb_syserr_new(NUM2INT(number), NIL_P(message) ? NULL : RSTRING_PTR(message));
}
static VALUE system_error_string(VALUE self, VALUE number, VALUE message) {
  return rb_syserr_new_str(NUM2INT(number), message);
}
static VALUE make(VALUE self, VALUE arguments) {
  return rb_make_exception(RARRAY_LENINT(arguments), RARRAY_PTR(arguments));
}
static VALUE classes(VALUE self) {
  VALUE values[5] = {rb_eStandardError, rb_eZeroDivError, rb_eEncCompatError, rb_mComparable, rb_eFatal};
  return rb_ary_new_from_values(5, values);
}

void Init_c_exceptions(void) {
  VALUE cls = rb_define_class("CExceptions", rb_cObject);
  rb_define_method(cls, "info", info, 0);
  rb_define_method(cls, "set_info", set_info, 1);
  rb_define_method(cls, "from_bytes", from_bytes, 1);
  rb_define_method(cls, "from_c_string", from_c_string, 1);
  rb_define_method(cls, "from_string", from_string, 1);
  rb_define_method(cls, "raise_it", raise_it, 1);
  rb_define_method(cls, "frozen", frozen, 1);
  rb_define_method(cls, "system_error", system_error, 2);
  rb_define_method(cls, "system_error_string", system_error_string, 2);
  rb_define_method(cls, "make", make, 1);
  rb_define_method(cls, "classes", classes, 0);
}
