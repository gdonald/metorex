#include "ruby.h"

static VALUE tagged_allocator(VALUE klass) {
  VALUE made = rb_get_alloc_func(rb_class_get_superclass(klass))(klass);
  rb_iv_set(made, "@tagged", Qtrue);
  return made;
}
static VALUE define_allocator(VALUE self, VALUE klass) {
  rb_define_alloc_func(klass, tagged_allocator);
  return klass;
}
static VALUE undefine_allocator(VALUE self, VALUE klass) {
  rb_undef_alloc_func(klass);
  return klass;
}
static VALUE allocator_kind(VALUE self, VALUE klass) {
  rb_alloc_func_t allocator = rb_get_alloc_func(klass);
  if (allocator == NULL) return ID2SYM(rb_intern("none"));
  return ID2SYM(rb_intern(allocator == tagged_allocator ? "tagged" : "default"));
}
static VALUE allocate(VALUE self, VALUE klass) { return rb_obj_alloc(klass); }
static VALUE duplicate(VALUE self, VALUE object) { return rb_obj_dup(object); }
static VALUE call_init(VALUE self, VALUE object, VALUE arguments) {
  rb_obj_call_init(object, RARRAY_LENINT(arguments), RARRAY_PTR(arguments));
  return object;
}
static VALUE class_of_object(VALUE self, VALUE object) { return rb_obj_class(object); }
static VALUE class_name(VALUE self, VALUE object) { return rb_str_new_cstr(rb_obj_classname(object)); }
static VALUE freeze(VALUE self, VALUE object) { return rb_obj_freeze(object); }
static VALUE frozen(VALUE self, VALUE object) { return rb_obj_frozen_p(object); }
static VALUE frozen_by_macro(VALUE self, VALUE object) { return RB_OBJ_FROZEN(object) ? Qtrue : Qfalse; }
static VALUE check_frozen(VALUE self, VALUE object) {
  rb_check_frozen(object);
  return Qtrue;
}
static VALUE object_id(VALUE self, VALUE object) { return rb_obj_id(object); }
static VALUE instance_of(VALUE self, VALUE object, VALUE klass) { return rb_obj_is_instance_of(object, klass); }
static VALUE kind_of(VALUE self, VALUE object, VALUE klass) { return rb_obj_is_kind_of(object, klass); }
static VALUE method_object(VALUE self, VALUE object, VALUE name) { return rb_obj_method(object, name); }
static VALUE method_arity(VALUE self, VALUE object, VALUE name) { return INT2FIX(rb_obj_method_arity(object, SYM2ID(name))); }
static VALUE responds(VALUE self, VALUE object, VALUE name) { return rb_respond_to(object, SYM2ID(name)) ? Qtrue : Qfalse; }
static VALUE responds_privately(VALUE self, VALUE object, VALUE name) {
  return rb_obj_respond_to(object, SYM2ID(name), 1) ? Qtrue : Qfalse;
}
static VALUE bound(VALUE self, VALUE klass, VALUE name, VALUE exclude_private) {
  return rb_method_boundp(klass, SYM2ID(name), RTEST(exclude_private)) ? Qtrue : Qfalse;
}
static VALUE special(VALUE self, VALUE object) { return rb_special_const_p(object); }
static VALUE able(VALUE self, VALUE object) { return FL_ABLE(object) ? Qtrue : Qfalse; }
static VALUE builtin_type(VALUE self, VALUE object) { return INT2FIX(BUILTIN_TYPE(object)); }
static VALUE to_id(VALUE self, VALUE name) { return ID2SYM(rb_to_id(name)); }
static VALUE check_convert(VALUE self, VALUE object, VALUE method) {
  return rb_check_convert_type(object, T_ARRAY, "Array", StringValueCStr(method));
}
static VALUE convert(VALUE self, VALUE object, VALUE method) {
  return rb_convert_type(object, T_ARRAY, "Array", StringValueCStr(method));
}
static VALUE check_array(VALUE self, VALUE object) { return rb_check_array_type(object); }
static VALUE check_string(VALUE self, VALUE object) { return rb_check_string_type(object); }
static VALUE check_integer(VALUE self, VALUE object, VALUE method) {
  return rb_check_to_integer(object, StringValueCStr(method));
}
static VALUE to_int(VALUE self, VALUE object) { return rb_to_int(object); }
static VALUE extend(VALUE self, VALUE object, VALUE module) {
  rb_extend_object(object, module);
  return object;
}
static VALUE instance_eval(VALUE self, VALUE object) { return rb_obj_instance_eval(0, NULL, object); }
static VALUE any_to_s(VALUE self, VALUE object) { return rb_any_to_s(object); }
static VALUE equal(VALUE self, VALUE first, VALUE second) { return rb_equal(first, second); }
static VALUE inherited(VALUE self, VALUE module, VALUE other) { return rb_class_inherited_p(module, other); }
static VALUE require_feature(VALUE self, VALUE feature) { return rb_require(StringValueCStr(feature)); }
static VALUE ivar_get(VALUE self, VALUE object, VALUE name) { return rb_ivar_get(object, SYM2ID(name)); }
static VALUE ivar_set(VALUE self, VALUE object, VALUE name, VALUE value) { return rb_ivar_set(object, SYM2ID(name), value); }
static VALUE ivar_defined(VALUE self, VALUE object, VALUE name) { return rb_ivar_defined(object, SYM2ID(name)); }
static VALUE iv_get(VALUE self, VALUE object, VALUE name) { return rb_iv_get(object, StringValueCStr(name)); }
static VALUE iv_set(VALUE self, VALUE object, VALUE name, VALUE value) {
  return rb_iv_set(object, StringValueCStr(name), value);
}
static VALUE attr_get(VALUE self, VALUE object, VALUE name) { return rb_attr_get(object, SYM2ID(name)); }
static VALUE instance_variables(VALUE self, VALUE object) { return rb_obj_instance_variables(object); }
static VALUE ivar_count(VALUE self, VALUE object) { return SIZET2NUM(rb_ivar_count(object)); }
static int collect_pair(ID name, VALUE value, VALUE pairs) {
  rb_ary_push(pairs, rb_ary_new_from_args(2, ID2SYM(name), value));
  return RARRAY_LEN(pairs) >= 3 ? ST_STOP : ST_CONTINUE;
}
static VALUE ivar_foreach(VALUE self, VALUE object) {
  VALUE pairs = rb_ary_new();
  rb_ivar_foreach(object, collect_pair, pairs);
  return pairs;
}
static VALUE copy_ivars(VALUE self, VALUE clone, VALUE object) {
  rb_copy_generic_ivar(clone, object);
  return clone;
}
static VALUE free_ivars(VALUE self, VALUE object) {
  rb_free_generic_ivar(object);
  return object;
}

void Init_c_objects(void) {
  VALUE cls = rb_define_class("CObjects", rb_cObject);
  rb_define_method(cls, "define_allocator", define_allocator, 1);
  rb_define_method(cls, "undefine_allocator", undefine_allocator, 1);
  rb_define_method(cls, "allocator_kind", allocator_kind, 1);
  rb_define_method(cls, "allocate", allocate, 1);
  rb_define_method(cls, "duplicate", duplicate, 1);
  rb_define_method(cls, "call_init", call_init, 2);
  rb_define_method(cls, "class_of_object", class_of_object, 1);
  rb_define_method(cls, "class_name", class_name, 1);
  rb_define_method(cls, "freeze", freeze, 1);
  rb_define_method(cls, "frozen", frozen, 1);
  rb_define_method(cls, "frozen_by_macro", frozen_by_macro, 1);
  rb_define_method(cls, "check_frozen", check_frozen, 1);
  rb_define_method(cls, "object_id_of", object_id, 1);
  rb_define_method(cls, "instance_of", instance_of, 2);
  rb_define_method(cls, "kind_of", kind_of, 2);
  rb_define_method(cls, "method_object", method_object, 2);
  rb_define_method(cls, "method_arity", method_arity, 2);
  rb_define_method(cls, "responds", responds, 2);
  rb_define_method(cls, "responds_privately", responds_privately, 2);
  rb_define_method(cls, "bound", bound, 3);
  rb_define_method(cls, "special", special, 1);
  rb_define_method(cls, "able", able, 1);
  rb_define_method(cls, "builtin_type", builtin_type, 1);
  rb_define_method(cls, "to_id", to_id, 1);
  rb_define_method(cls, "check_convert", check_convert, 2);
  rb_define_method(cls, "convert", convert, 2);
  rb_define_method(cls, "check_array", check_array, 1);
  rb_define_method(cls, "check_string", check_string, 1);
  rb_define_method(cls, "check_integer", check_integer, 2);
  rb_define_method(cls, "to_int", to_int, 1);
  rb_define_method(cls, "extend", extend, 2);
  rb_define_method(cls, "instance_eval_in", instance_eval, 1);
  rb_define_method(cls, "any_to_s", any_to_s, 1);
  rb_define_method(cls, "equal", equal, 2);
  rb_define_method(cls, "inherited", inherited, 2);
  rb_define_method(cls, "require_feature", require_feature, 1);
  rb_define_method(cls, "ivar_get", ivar_get, 2);
  rb_define_method(cls, "ivar_set", ivar_set, 3);
  rb_define_method(cls, "ivar_defined", ivar_defined, 2);
  rb_define_method(cls, "iv_get", iv_get, 2);
  rb_define_method(cls, "iv_set", iv_set, 3);
  rb_define_method(cls, "attr_get", attr_get, 2);
  rb_define_method(cls, "instance_variables_of", instance_variables, 1);
  rb_define_method(cls, "ivar_count", ivar_count, 1);
  rb_define_method(cls, "ivar_foreach", ivar_foreach, 1);
  rb_define_method(cls, "copy_ivars", copy_ivars, 2);
  rb_define_method(cls, "free_ivars", free_ivars, 1);
  rb_define_method(cls, "unavailable", rb_f_notimplement, -1);
}
