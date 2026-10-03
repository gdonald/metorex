#include "ruby.h"

static VALUE make(VALUE self, VALUE first, VALUE last, VALUE exclusive) {
  return rb_range_new(first, last, RTEST(exclusive));
}
static VALUE ends(VALUE self, VALUE range) {
  VALUE first = Qnil, last = Qnil;
  int exclusive = 0;
  if (!rb_range_values(range, &first, &last, &exclusive)) return Qfalse;
  VALUE values[3] = {first, last, exclusive ? Qtrue : Qfalse};
  return rb_ary_new_from_values(3, values);
}
static VALUE start_length(VALUE self, VALUE range, VALUE length, VALUE err) {
  long start = -1, covered = -1;
  VALUE answer = rb_range_beg_len(range, &start, &covered, FIX2LONG(length), FIX2INT(err));
  VALUE values[3] = {answer, LONG2NUM(start), LONG2NUM(covered)};
  return rb_ary_new_from_values(3, values);
}
static VALUE parts(VALUE self, VALUE sequence) {
  rb_arithmetic_sequence_components_t held;
  if (!rb_arithmetic_sequence_extract(sequence, &held)) return Qfalse;
  VALUE values[4] = {held.begin, held.end, held.step, held.exclude_end ? Qtrue : Qfalse};
  return rb_ary_new_from_values(4, values);
}
static VALUE start_length_step(VALUE self, VALUE sequence, VALUE length, VALUE err) {
  long start = -1, covered = -1, step = 0;
  VALUE answer = rb_arithmetic_sequence_beg_len_step(sequence, &start, &covered, &step, FIX2LONG(length), FIX2INT(err));
  VALUE values[4] = {answer, LONG2NUM(start), LONG2NUM(covered), LONG2NUM(step)};
  return rb_ary_new_from_values(4, values);
}
static VALUE store(VALUE self, VALUE array, VALUE index, VALUE element) {
  rb_ary_store(array, FIX2LONG(index), element);
  return array;
}

void Init_c_ranges(void) {
  VALUE cls = rb_define_class("CRanges", rb_cObject);
  rb_define_method(cls, "make", make, 3);
  rb_define_method(cls, "ends", ends, 1);
  rb_define_method(cls, "start_length", start_length, 3);
  rb_define_method(cls, "parts", parts, 1);
  rb_define_method(cls, "start_length_step", start_length_step, 3);
  rb_define_method(cls, "store", store, 3);
}
