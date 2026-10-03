#include "ruby.h"

static VALUE given(VALUE self) { return rb_block_given_p() ? Qtrue : Qfalse; }
static VALUE yield_one(VALUE self, VALUE value) { return rb_yield(value); }
static VALUE yield_none(VALUE self) { return rb_yield(Qundef); }
static VALUE yield_two(VALUE self, VALUE first, VALUE second) { return rb_yield_values(2, first, second); }
static VALUE yield_splat(VALUE self, VALUE values) { return rb_yield_splat(values); }
static VALUE yield_then(VALUE self) {
  rb_yield(INT2FIX(1));
  return ID2SYM(rb_intern("finished"));
}

static int keep_odd(VALUE element, VALUE seen) {
  rb_funcall(seen, rb_intern("push"), 1, element);
  if (FIX2LONG(element) == 4) return ST_STOP;
  return FIX2LONG(element) % 2 == 0 ? ST_DELETE : ST_CONTINUE;
}
static VALUE walk(VALUE self, VALUE set, VALUE seen) {
  rb_set_foreach(set, keep_odd, seen);
  return set;
}
static VALUE set_calls(VALUE self) {
  VALUE set = rb_set_new_capa(4);
  VALUE answers[8];
  answers[0] = rb_set_add(set, INT2FIX(1)) ? Qtrue : Qfalse;
  answers[1] = rb_set_add(set, INT2FIX(1)) ? Qtrue : Qfalse;
  answers[2] = rb_set_lookup(set, INT2FIX(1)) ? Qtrue : Qfalse;
  answers[3] = rb_set_lookup(set, INT2FIX(2)) ? Qtrue : Qfalse;
  answers[4] = SIZET2NUM(rb_set_size(set));
  answers[5] = rb_set_delete(set, INT2FIX(1)) ? Qtrue : Qfalse;
  answers[6] = rb_set_delete(set, INT2FIX(1)) ? Qtrue : Qfalse;
  rb_set_add(set, INT2FIX(3));
  answers[7] = rb_set_clear(set);
  return rb_ary_new_from_values(8, answers);
}
static VALUE empty_set(VALUE self) { return rb_set_new(); }

static VALUE conversions(VALUE self) {
  VALUE values[6] = {LONG2NUM(-5), ULONG2NUM(18446744073709551615UL), LL2NUM(-9223372036854775807LL - 1),
                     ULL2NUM(9223372036854775808ULL), SIZET2NUM(7), SSIZET2NUM(-7)};
  return rb_ary_new_from_values(6, values);
}

void Init_c_blocks(void) {
  VALUE cls = rb_define_class("CBlocks", rb_cObject);
  rb_define_method(cls, "given", given, 0);
  rb_define_method(cls, "yield_one", yield_one, 1);
  rb_define_method(cls, "yield_none", yield_none, 0);
  rb_define_method(cls, "yield_two", yield_two, 2);
  rb_define_method(cls, "yield_splat", yield_splat, 1);
  rb_define_method(cls, "yield_then", yield_then, 0);
  rb_define_method(cls, "walk", walk, 2);
  rb_define_method(cls, "set_calls", set_calls, 0);
  rb_define_method(cls, "empty_set", empty_set, 0);
  rb_define_method(cls, "conversions", conversions, 0);
}
