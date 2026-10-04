#include "ruby.h"

static VALUE convert(VALUE self, VALUE object) { return rb_Array(object); }
static VALUE capacity(VALUE self, VALUE size) { return rb_ary_new_capa(NUM2LONG(size)); }
static VALUE triple(VALUE self, VALUE first, VALUE second, VALUE third) {
  VALUE values[3] = {first, second, third};
  return rb_ary_new4(3, values);
}
static VALUE triple_args(VALUE self, VALUE first, VALUE second, VALUE third) {
  return rb_ary_new3(3, first, second, third);
}
static VALUE element(VALUE self, VALUE array, VALUE index) { return RARRAY_AREF(array, NUM2LONG(index)); }
static VALUE set_element(VALUE self, VALUE array, VALUE index, VALUE value) {
  RARRAY_ASET(array, NUM2LONG(index), value);
  return array;
}
static VALUE fill(VALUE self, VALUE array, VALUE value) {
  VALUE *elements = RARRAY_PTR(array);
  long index;
  for (index = 0; index < RARRAY_LEN(array); index++) elements[index] = value;
  return array;
}
static VALUE copy_into(VALUE self, VALUE from, VALUE to) {
  memcpy(RARRAY_PTR(to), RARRAY_PTR(from), RARRAY_LEN(from) * sizeof(VALUE));
  return to;
}
static VALUE aref(int argc, VALUE *argv, VALUE self) { return rb_ary_aref(argc - 1, argv + 1, argv[0]); }
static VALUE cat(int argc, VALUE *argv, VALUE self) { return rb_ary_cat(argv[0], argv + 1, argc - 1); }
static VALUE clear(VALUE self, VALUE array) { return rb_ary_clear(array); }
static VALUE concat(VALUE self, VALUE array, VALUE other) { return rb_ary_concat(array, other); }
static VALUE delete_element(VALUE self, VALUE array, VALUE element) { return rb_ary_delete(array, element); }
static VALUE delete_at(VALUE self, VALUE array, VALUE index) { return rb_ary_delete_at(array, NUM2LONG(index)); }
static VALUE freeze(VALUE self, VALUE array) { return rb_ary_freeze(array); }
static VALUE includes(VALUE self, VALUE array, VALUE element) { return rb_ary_includes(array, element); }
static VALUE join(VALUE self, VALUE array, VALUE separator) { return rb_ary_join(array, separator); }
static VALUE plus(VALUE self, VALUE array, VALUE other) { return rb_ary_plus(array, other); }
static VALUE reverse(VALUE self, VALUE array) { return rb_ary_reverse(array); }
static VALUE rotate(VALUE self, VALUE array, VALUE count) { return rb_ary_rotate(array, NUM2LONG(count)); }
static VALUE shift(VALUE self, VALUE array) { return rb_ary_shift(array); }
static VALUE sort(VALUE self, VALUE array) { return rb_ary_sort(array); }
static VALUE sort_bang(VALUE self, VALUE array) { return rb_ary_sort_bang(array); }
static VALUE subseq(VALUE self, VALUE array, VALUE start, VALUE length) {
  return rb_ary_subseq(array, NUM2LONG(start), NUM2LONG(length));
}
static VALUE to_ary(VALUE self, VALUE object) { return rb_ary_to_ary(object); }
static VALUE to_s(VALUE self, VALUE array) { return rb_ary_to_s(array); }
static VALUE pair(VALUE self, VALUE first, VALUE second) { return rb_assoc_new(first, second); }
static VALUE cleared(VALUE self, VALUE first, VALUE second) {
  VALUE values[2] = {first, second};
  rb_mem_clear(values, 2);
  return rb_ary_new4(2, values);
}

static VALUE collect(RB_BLOCK_CALL_FUNC_ARGLIST(yielded, collected)) {
  return rb_ary_push(collected, yielded);
}
static VALUE collect_each(VALUE self, VALUE object, VALUE name) {
  VALUE collected = rb_ary_new();
  rb_block_call(object, rb_intern_str(name), 0, 0, collect, collected);
  return collected;
}
static VALUE pass_on(RB_BLOCK_CALL_FUNC_ARGLIST(yielded, unused)) {
  return rb_yield(yielded);
}
static VALUE yield_each(VALUE self, VALUE object) {
  return rb_block_call(object, rb_intern("each"), 0, 0, pass_on, Qnil);
}
static VALUE each_slice(VALUE self, VALUE object, VALUE size) {
  VALUE collected = rb_ary_new();
  rb_block_call(object, rb_intern("each_slice"), 1, &size, collect, collected);
  return collected;
}

void Init_c_arrays(void) {
  VALUE cls = rb_define_class("CArrays", rb_cObject);
  rb_define_method(cls, "convert", convert, 1);
  rb_define_method(cls, "capacity", capacity, 1);
  rb_define_method(cls, "triple", triple, 3);
  rb_define_method(cls, "triple_args", triple_args, 3);
  rb_define_method(cls, "element", element, 2);
  rb_define_method(cls, "set_element", set_element, 3);
  rb_define_method(cls, "fill", fill, 2);
  rb_define_method(cls, "copy_into", copy_into, 2);
  rb_define_method(cls, "aref", aref, -1);
  rb_define_method(cls, "cat", cat, -1);
  rb_define_method(cls, "clear", clear, 1);
  rb_define_method(cls, "concat", concat, 2);
  rb_define_method(cls, "delete_element", delete_element, 2);
  rb_define_method(cls, "delete_at", delete_at, 2);
  rb_define_method(cls, "freeze", freeze, 1);
  rb_define_method(cls, "includes", includes, 2);
  rb_define_method(cls, "join", join, 2);
  rb_define_method(cls, "plus", plus, 2);
  rb_define_method(cls, "reverse", reverse, 1);
  rb_define_method(cls, "rotate", rotate, 2);
  rb_define_method(cls, "shift", shift, 1);
  rb_define_method(cls, "sort", sort, 1);
  rb_define_method(cls, "sort_bang", sort_bang, 1);
  rb_define_method(cls, "subseq", subseq, 3);
  rb_define_method(cls, "to_ary", to_ary, 1);
  rb_define_method(cls, "to_s", to_s, 1);
  rb_define_method(cls, "pair", pair, 2);
  rb_define_method(cls, "cleared", cleared, 2);
  rb_define_method(cls, "collect_each", collect_each, 2);
  rb_define_method(cls, "yield_each", yield_each, 1);
  rb_define_method(cls, "each_slice", each_slice, 2);
}
