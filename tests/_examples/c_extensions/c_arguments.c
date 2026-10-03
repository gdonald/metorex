#include "ruby.h"

static VALUE pair(VALUE first, VALUE second) {
  VALUE values[2] = {first, second};
  return rb_ary_new_from_values(2, values);
}

static VALUE leading_optional(int count, VALUE *arguments, VALUE self) {
  VALUE first, second;
  rb_scan_args(count, arguments, "11", &first, &second);
  return pair(first, second);
}
static VALUE leading_only(int count, VALUE *arguments, VALUE self) {
  VALUE first, second;
  rb_scan_args(count, arguments, "2", &first, &second);
  return pair(first, second);
}
static VALUE splat_trailing(int count, VALUE *arguments, VALUE self) {
  VALUE first, rest, last;
  rb_scan_args(count, arguments, "1*1", &first, &rest, &last);
  VALUE values[3] = {first, rest, last};
  return rb_ary_new_from_values(3, values);
}
static VALUE optional_only(int count, VALUE *arguments, VALUE self) {
  VALUE first, second;
  rb_scan_args(count, arguments, "02", &first, NULL);
  rb_scan_args(count, arguments, "02", NULL, &second);
  return pair(first, second);
}
static VALUE splat_only(int count, VALUE *arguments, VALUE self) {
  VALUE rest;
  rb_scan_args(count, arguments, "*", &rest);
  rb_scan_args(count, arguments, "*", NULL);
  return rest;
}
static VALUE trailing_ignored(int count, VALUE *arguments, VALUE self) {
  VALUE last;
  rb_scan_args(count, arguments, "*1", NULL, &last);
  rb_scan_args(count, arguments, "*1", NULL, NULL);
  return last;
}
static VALUE bad_format(int count, VALUE *arguments, VALUE self) {
  rb_scan_args(count, arguments, "1x", NULL);
  return Qnil;
}

static VALUE elements(VALUE self, VALUE array) {
  VALUE *values = RARRAY_PTR(array);
  return rb_ary_new_from_values(RARRAY_LEN(array), values);
}
static VALUE same_pointer(VALUE self, VALUE array) {
  return RARRAY_PTR(array) == RARRAY_PTR(array) ? Qtrue : Qfalse;
}
static VALUE empty(VALUE self) { return rb_ary_new_from_values(0, NULL); }

static VALUE sizing(VALUE object, VALUE arguments, VALUE enumerator) {
  VALUE values[3] = {object, arguments, enumerator};
  return rb_ary_new_from_values(3, values);
}
static VALUE enumerate(int count, VALUE *arguments, VALUE self) {
  VALUE object, name, rest;
  rb_scan_args(count, arguments, "2*", &object, &name, &rest);
  return rb_enumeratorize(object, name, (int)RARRAY_LEN(rest), RARRAY_PTR(rest));
}
static VALUE enumerate_sized(int count, VALUE *arguments, VALUE self) {
  VALUE object, name, rest;
  rb_scan_args(count, arguments, "2*", &object, &name, &rest);
  return rb_enumeratorize_with_size(object, name, (int)RARRAY_LEN(rest), RARRAY_PTR(rest), sizing);
}
static VALUE enumerate_unsized(VALUE self, VALUE object, VALUE name) {
  return rb_enumeratorize_with_size(object, name, 0, NULL, NULL);
}

void Init_c_arguments(void) {
  VALUE cls = rb_define_class("CArguments", rb_cObject);
  rb_define_method(cls, "leading_optional", leading_optional, -1);
  rb_define_method(cls, "leading_only", leading_only, -1);
  rb_define_method(cls, "splat_trailing", splat_trailing, -1);
  rb_define_method(cls, "optional_only", optional_only, -1);
  rb_define_method(cls, "splat_only", splat_only, -1);
  rb_define_method(cls, "trailing_ignored", trailing_ignored, -1);
  rb_define_method(cls, "bad_format", bad_format, -1);
  rb_define_method(cls, "elements", elements, 1);
  rb_define_method(cls, "same_pointer", same_pointer, 1);
  rb_define_method(cls, "empty", empty, 0);
  rb_define_method(cls, "enumerate", enumerate, -1);
  rb_define_method(cls, "enumerate_sized", enumerate_sized, -1);
  rb_define_method(cls, "enumerate_unsized", enumerate_unsized, 2);
}
