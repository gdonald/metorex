#include "ruby.h"
#include <stdlib.h>

struct counter {
  int count;
};

static size_t counter_size(const void *data) { return 100 + ((const struct counter *)data)->count; }

static const rb_data_type_t base_type = {"base", {NULL, free, NULL}};
static const rb_data_type_t counter_type = {"counter", {NULL, free, counter_size}, &base_type};
static const rb_data_type_t other_type = {"other", {NULL, free, NULL}};

static VALUE allocate_counter(VALUE klass) {
  struct counter *made;
  VALUE object = TypedData_Make_Struct(klass, struct counter, &counter_type, made);
  made->count = 5;
  return object;
}

static VALUE count(VALUE self) {
  struct counter *held;
  TypedData_Get_Struct(self, struct counter, &counter_type, held);
  return INT2FIX(held->count);
}

static VALUE increment(VALUE self) {
  struct counter *held;
  TypedData_Get_Struct(self, struct counter, &counter_type, held);
  held->count++;
  return self;
}

static VALUE wrap_typed(VALUE self, VALUE start) {
  struct counter *made = malloc(sizeof(struct counter));
  made->count = FIX2INT(start);
  return TypedData_Wrap_Struct(rb_cObject, &counter_type, made);
}

static VALUE wrap_untyped(VALUE self, VALUE start) {
  int *made = malloc(sizeof(int));
  *made = FIX2INT(start);
  return Data_Wrap_Struct(rb_cObject, NULL, free, made);
}

static VALUE make_untyped(VALUE self, VALUE start) {
  int *made;
  VALUE object = Data_Make_Struct(rb_cObject, int, NULL, free, made);
  *made = FIX2INT(start);
  return object;
}

static VALUE untyped_value(VALUE self, VALUE object) {
  int *held;
  Data_Get_Struct(object, int, held);
  return INT2FIX(*held);
}

static VALUE as_base(VALUE self, VALUE object) {
  struct counter *held = rb_check_typeddata(object, &base_type);
  return INT2FIX(held->count);
}

static VALUE as_other(VALUE self, VALUE object) {
  rb_check_typeddata(object, &other_type);
  return Qtrue;
}

static VALUE replace(VALUE self, VALUE object, VALUE start) {
  struct counter *made = malloc(sizeof(struct counter));
  made->count = FIX2INT(start);
  free(RTYPEDDATA(object)->data);
  RTYPEDDATA(object)->data = made;
  return object;
}

static VALUE describe(VALUE self, VALUE object) {
  return rb_ary_new_from_args(4, RTYPEDDATA_P(object) ? Qtrue : Qfalse,
                              RTYPEDDATA_P(object) ? rb_str_new_cstr(RTYPEDDATA_TYPE(object)->wrap_struct_name) : Qnil,
                              rb_typeddata_is_kind_of(object, &base_type) ? Qtrue : Qfalse,
                              TYPE(object) == T_DATA ? Qtrue : Qfalse);
}

static VALUE check_type(VALUE self, VALUE object, VALUE type) {
  rb_check_type(object, FIX2INT(type));
  return Qtrue;
}

static VALUE data_ptr_of(VALUE self, VALUE object) {
  return DATA_PTR(object) != NULL ? Qtrue : Qfalse;
}

void Init_c_data(void) {
  VALUE counter = rb_define_class("CDataCounter", rb_cObject);
  rb_define_alloc_func(counter, allocate_counter);
  rb_define_method(counter, "count", count, 0);
  rb_define_method(counter, "increment", increment, 0);

  VALUE data = rb_define_class("CData", rb_cObject);
  rb_define_method(data, "wrap_typed", wrap_typed, 1);
  rb_define_method(data, "wrap_untyped", wrap_untyped, 1);
  rb_define_method(data, "make_untyped", make_untyped, 1);
  rb_define_method(data, "untyped_value", untyped_value, 1);
  rb_define_method(data, "as_base", as_base, 1);
  rb_define_method(data, "as_other", as_other, 1);
  rb_define_method(data, "replace", replace, 2);
  rb_define_method(data, "describe", describe, 1);
  rb_define_method(data, "check_type", check_type, 2);
  rb_define_method(data, "data_ptr_of", data_ptr_of, 1);
  rb_define_const(data, "T_DATA", INT2FIX(T_DATA));
  rb_define_const(data, "T_STRING", INT2FIX(T_STRING));
}
