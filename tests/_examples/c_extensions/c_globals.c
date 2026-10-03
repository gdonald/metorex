#include "ruby.h"

static VALUE stored = Qnil;
static VALUE fixed = Qnil;
static VALUE doubled_value = Qnil;
static long counter = 0;

static VALUE doubled_getter(ID id, VALUE *data) { return *data; }
static void doubled_setter(VALUE value, ID id, VALUE *data) { *data = INT2FIX(FIX2LONG(value) * 2); }
static VALUE counting_getter(ID id, VALUE *data) { return rb_ary_new_from_args(2, ID2SYM(id), LONG2NUM(counter++)); }
static void counting_setter(VALUE value, ID id, VALUE *data) { counter = FIX2LONG(value); }

static VALUE define_all(VALUE self) {
  stored = rb_str_new_cstr("stored in C");
  rb_define_variable("$c_stored", &stored);
  fixed = INT2FIX(15);
  rb_define_readonly_variable("c_fixed", &fixed);
  rb_define_hooked_variable("$c_doubled", &doubled_value, doubled_getter, doubled_setter);
  rb_define_hooked_variable("$c_defaults", &stored, NULL, NULL);
  rb_define_hooked_variable("$c_nowhere", NULL, NULL, NULL);
  rb_define_virtual_variable("$c_virtual", NULL, NULL);
  rb_define_virtual_variable("$c_counting", counting_getter, counting_setter);
  return Qnil;
}
static VALUE stored_now(VALUE self) { return stored; }
static VALUE get(VALUE self, VALUE name) { return rb_gv_get(RSTRING_PTR(name)); }
static VALUE set(VALUE self, VALUE name, VALUE value) { return rb_gv_set(RSTRING_PTR(name), value); }
static VALUE names(VALUE self) { return rb_f_global_variables(); }
static VALUE last_line(VALUE self, VALUE line) {
  rb_lastline_set(line);
  return rb_lastline_get();
}
static VALUE separators(VALUE self) {
  return rb_ary_new_from_args(5, rb_fs, rb_rs, rb_output_fs, rb_output_rs, rb_default_rs);
}
static VALUE streams(VALUE self) { return rb_ary_new_from_args(4, rb_stdin, rb_stdout, rb_stderr, rb_defout); }

void Init_c_globals(void) {
  VALUE cls = rb_define_class("CGlobals", rb_cObject);
  rb_define_method(cls, "define_all", define_all, 0);
  rb_define_method(cls, "stored_now", stored_now, 0);
  rb_define_method(cls, "get", get, 1);
  rb_define_method(cls, "set", set, 2);
  rb_define_method(cls, "names", names, 0);
  rb_define_method(cls, "last_line", last_line, 1);
  rb_define_method(cls, "separators", separators, 0);
  rb_define_method(cls, "streams", streams, 0);
}
