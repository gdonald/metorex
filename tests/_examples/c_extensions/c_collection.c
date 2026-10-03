#include "ruby.h"

static VALUE kept;
static VALUE marked;

static VALUE keep(VALUE self, VALUE value) {
  rb_gc_register_address(&kept);
  kept = value;
  rb_global_variable(&marked);
  marked = rb_ary_new_from_args(1, value);
  rb_gc_register_mark_object(value);
  rb_gc_adjust_memory_usage(64);
  rb_gc_adjust_memory_usage(-64);
  return Qnil;
}
static VALUE release(VALUE self) {
  rb_gc_unregister_address(&kept);
  return Qnil;
}
static VALUE held(VALUE self) { return rb_ary_new_from_args(2, kept, marked); }
static VALUE switched(VALUE self) {
  VALUE first = rb_gc_disable();
  VALUE second = rb_gc_disable();
  VALUE third = rb_gc_enable();
  VALUE fourth = rb_gc_enable();
  return rb_ary_new_from_args(4, first, second, third, fourth);
}
static VALUE runs(VALUE self) {
  size_t before = rb_gc_count();
  rb_gc();
  rb_gc_start();
  return SIZET2NUM(rb_gc_count() - before);
}
static VALUE latest(VALUE self, VALUE key) { return rb_gc_latest_gc_info(key); }
static VALUE sizes(VALUE self, VALUE value) {
  return rb_ary_new_from_args(2, SSIZET2NUM(NUM2SSIZET(value)), SIZET2NUM(NUM2SIZET(value)));
}

void Init_c_collection(void) {
  VALUE cls = rb_define_class("CCollection", rb_cObject);
  rb_define_method(cls, "keep", keep, 1);
  rb_define_method(cls, "release", release, 0);
  rb_define_method(cls, "held", held, 0);
  rb_define_method(cls, "switched", switched, 0);
  rb_define_method(cls, "runs", runs, 0);
  rb_define_method(cls, "latest", latest, 1);
  rb_define_method(cls, "sizes", sizes, 1);
}
