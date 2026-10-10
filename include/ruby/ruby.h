#ifndef RUBY_RUBY_H
#define RUBY_RUBY_H 1

/* A VALUE is a fixnum when its low bit is set, one of the four special
 * constants below, or a handle metorex hands out for any other object. */

#include <alloca.h>
#include <limits.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <unistd.h>
#include <sys/types.h>
#include <time.h>
/* The headers MRI's ruby.h brings in, which an extension written for it
   relies on having without including them itself. */
#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <math.h>
#include <strings.h>
#include <sys/stat.h>

#include "ruby/st.h"

#define NORETURN(x) __attribute__((__noreturn__)) x
#define UNREACHABLE_RETURN(value) __builtin_unreachable()
#define UNREACHABLE __builtin_unreachable()

#ifdef __cplusplus
extern "C" {
#define ANYARGS ...
#else
#define ANYARGS
#endif

#define SIZEOF_INT __SIZEOF_INT__
#define SIZEOF_SHORT __SIZEOF_SHORT__
#define SIZEOF_LONG __SIZEOF_LONG__
#define SIZEOF_LONG_LONG __SIZEOF_LONG_LONG__
#define SIZEOF_VOIDP __SIZEOF_POINTER__
#define SIZEOF_SIZE_T __SIZEOF_SIZE_T__
#define SIZEOF_DOUBLE __SIZEOF_DOUBLE__
#define SIZEOF_TIME_T __SIZEOF_LONG__
#define SIZEOF_VALUE SIZEOF_VOIDP
#define HAVE_LONG_LONG 1
#define HAVE_UNISTD_H 1
#define LONG_LONG long long

typedef uintptr_t VALUE;
typedef uintptr_t ID;
typedef intptr_t SIGNED_VALUE;

#define RUBY_EXTERN extern

#define Qfalse ((VALUE)0x00)
#define Qnil ((VALUE)0x08)
#define Qtrue ((VALUE)0x14)
#define Qundef ((VALUE)0x24)

#define RTEST(v) (((VALUE)(v) & ~Qnil) != 0)
#define NIL_P(v) ((VALUE)(v) == Qnil)

#define RUBY_FIXNUM_FLAG 0x01
#define FIXNUM_P(v) ((((SIGNED_VALUE)(v)) & RUBY_FIXNUM_FLAG) != 0)
#define INT2FIX(i) ((((VALUE)(SIGNED_VALUE)(i)) << 1) | RUBY_FIXNUM_FLAG)
#define LONG2FIX(i) INT2FIX(i)
#define FIX2LONG(v) ((long)(((SIGNED_VALUE)(v)) >> 1))
#define INT2NUM(i) INT2FIX(i)
#define UINT2NUM(i) LONG2FIX((unsigned int)(i))

VALUE rb_int2inum(intptr_t number);
VALUE rb_uint2inum(uintptr_t number);
VALUE rb_ll2inum(long long number);
VALUE rb_ull2inum(unsigned long long number);
#define LONG2NUM(v) rb_int2inum((intptr_t)(v))
#define ULONG2NUM(v) rb_uint2inum((uintptr_t)(v))
#define LL2NUM(v) rb_ll2inum((long long)(v))
#define ULL2NUM(v) rb_ull2inum((unsigned long long)(v))
#define SIZET2NUM(v) ULONG2NUM(v)
#define SSIZET2NUM(v) LONG2NUM(v)

long rb_num2long(VALUE value);
unsigned long rb_num2ulong(VALUE value);
long rb_num2int(VALUE value);
long rb_fix2int(VALUE value);
unsigned long rb_num2uint(VALUE value);
unsigned long rb_fix2uint(VALUE value);
#define NUM2LONG(v) rb_num2long((VALUE)(v))
#define NUM2ULONG(v) rb_num2ulong((VALUE)(v))
#define NUM2INT(v) ((int)rb_num2int((VALUE)(v)))
#define NUM2UINT(v) ((unsigned int)rb_num2uint((VALUE)(v)))
#define FIX2INT(v) ((int)rb_fix2int((VALUE)(v)))
#define FIX2UINT(v) ((unsigned int)rb_fix2uint((VALUE)(v)))

ID rb_intern(const char *name);
VALUE rb_id2sym(ID id);
ID rb_sym2id(VALUE symbol);
const char *rb_id2name(ID id);
ID rb_intern2(const char *text, long length);
ID rb_intern_str(VALUE string);
#define rb_intern_const(text) rb_intern(text)
VALUE rb_id2str(ID id);
VALUE rb_sym2str(VALUE symbol);
VALUE rb_to_symbol(VALUE value);
int rb_symbol_p(VALUE value);
#define SYMBOL_P(v) rb_symbol_p((VALUE)(v))
#define RB_SYMBOL_P(v) SYMBOL_P(v)
int rb_is_const_id(ID id);
int rb_is_instance_id(ID id);
int rb_is_class_id(ID id);
#define ID2SYM(id) rb_id2sym((ID)(id))
#define SYM2ID(v) rb_sym2id((VALUE)(v))

RUBY_EXTERN VALUE rb_mKernel;
RUBY_EXTERN VALUE rb_mComparable;
RUBY_EXTERN VALUE rb_mEnumerable;
RUBY_EXTERN VALUE rb_mErrno;
RUBY_EXTERN VALUE rb_mFileTest;
RUBY_EXTERN VALUE rb_mGC;
RUBY_EXTERN VALUE rb_mMath;
RUBY_EXTERN VALUE rb_mProcess;
RUBY_EXTERN VALUE rb_mWaitReadable;
RUBY_EXTERN VALUE rb_mWaitWritable;
RUBY_EXTERN VALUE rb_cBasicObject;
RUBY_EXTERN VALUE rb_cObject;
RUBY_EXTERN VALUE rb_cArray;
RUBY_EXTERN VALUE rb_cBinding;
RUBY_EXTERN VALUE rb_cClass;
RUBY_EXTERN VALUE rb_cComplex;
RUBY_EXTERN VALUE rb_cDir;
RUBY_EXTERN VALUE rb_cEncoding;
RUBY_EXTERN VALUE rb_cEnumerator;
RUBY_EXTERN VALUE rb_cFalseClass;
RUBY_EXTERN VALUE rb_cFile;
RUBY_EXTERN VALUE rb_cFloat;
RUBY_EXTERN VALUE rb_cHash;
RUBY_EXTERN VALUE rb_cIO;
RUBY_EXTERN VALUE rb_cInteger;
RUBY_EXTERN VALUE rb_cMatch;
RUBY_EXTERN VALUE rb_cMethod;
RUBY_EXTERN VALUE rb_cModule;
RUBY_EXTERN VALUE rb_cNilClass;
RUBY_EXTERN VALUE rb_cNumeric;
RUBY_EXTERN VALUE rb_cProc;
RUBY_EXTERN VALUE rb_cRandom;
RUBY_EXTERN VALUE rb_cRange;
RUBY_EXTERN VALUE rb_cRational;
RUBY_EXTERN VALUE rb_cRegexp;
RUBY_EXTERN VALUE rb_cSet;
RUBY_EXTERN VALUE rb_cStat;
RUBY_EXTERN VALUE rb_cString;
RUBY_EXTERN VALUE rb_cStruct;
RUBY_EXTERN VALUE rb_cSymbol;
RUBY_EXTERN VALUE rb_cThread;
RUBY_EXTERN VALUE rb_cTime;
RUBY_EXTERN VALUE rb_cTrueClass;
RUBY_EXTERN VALUE rb_cUnboundMethod;
RUBY_EXTERN VALUE rb_eException;
RUBY_EXTERN VALUE rb_eFatal;
RUBY_EXTERN VALUE rb_eStandardError;
RUBY_EXTERN VALUE rb_eSystemExit;
RUBY_EXTERN VALUE rb_eInterrupt;
RUBY_EXTERN VALUE rb_eSignal;
RUBY_EXTERN VALUE rb_eArgError;
RUBY_EXTERN VALUE rb_eEOFError;
RUBY_EXTERN VALUE rb_eIndexError;
RUBY_EXTERN VALUE rb_eStopIteration;
RUBY_EXTERN VALUE rb_eKeyError;
RUBY_EXTERN VALUE rb_eRangeError;
RUBY_EXTERN VALUE rb_eIOError;
RUBY_EXTERN VALUE rb_eRuntimeError;
RUBY_EXTERN VALUE rb_eFrozenError;
RUBY_EXTERN VALUE rb_eSecurityError;
RUBY_EXTERN VALUE rb_eSystemCallError;
RUBY_EXTERN VALUE rb_eThreadError;
RUBY_EXTERN VALUE rb_eTypeError;
RUBY_EXTERN VALUE rb_eZeroDivError;
RUBY_EXTERN VALUE rb_eNotImpError;
RUBY_EXTERN VALUE rb_eNoMemError;
RUBY_EXTERN VALUE rb_eNoMethodError;
RUBY_EXTERN VALUE rb_eFloatDomainError;
RUBY_EXTERN VALUE rb_eLocalJumpError;
RUBY_EXTERN VALUE rb_eSysStackError;
RUBY_EXTERN VALUE rb_eRegexpError;
RUBY_EXTERN VALUE rb_eEncodingError;
RUBY_EXTERN VALUE rb_eEncCompatError;
RUBY_EXTERN VALUE rb_eNoMatchingPatternError;
RUBY_EXTERN VALUE rb_eNoMatchingPatternKeyError;
RUBY_EXTERN VALUE rb_eScriptError;
RUBY_EXTERN VALUE rb_eNameError;
RUBY_EXTERN VALUE rb_eSyntaxError;
RUBY_EXTERN VALUE rb_eLoadError;
RUBY_EXTERN VALUE rb_eMathDomainError;

VALUE rb_class_of(VALUE object);
VALUE rb_singleton_class(VALUE object);
#define CLASS_OF(v) rb_class_of((VALUE)(v))
#define RBASIC_CLASS(v) rb_class_of((VALUE)(v))

VALUE rb_define_class(const char *name, VALUE super);
VALUE rb_define_class_under(VALUE outer, const char *name, VALUE super);
VALUE rb_define_class_id_under(VALUE outer, ID name, VALUE super);
VALUE rb_define_module(const char *name);
VALUE rb_define_module_under(VALUE outer, const char *name);
void rb_define_method(VALUE klass, const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_define_const(VALUE module, const char *name, VALUE value);

VALUE rb_funcallv(VALUE receiver, ID name, int count, const VALUE *arguments);
VALUE rb_const_get(VALUE module, ID name);
VALUE rb_const_get_at(VALUE module, ID name);
VALUE rb_const_get_from(VALUE module, ID name);
int rb_const_defined(VALUE module, ID name);
int rb_const_defined_at(VALUE module, ID name);
void rb_const_set(VALUE module, ID name, VALUE value);
void rb_define_global_const(const char *name, VALUE value);
void rb_define_alias(VALUE module, const char *new_name, const char *old_name);
void rb_alias(VALUE module, ID new_name, ID old_name);
void rb_define_private_method(VALUE module, const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_define_protected_method(VALUE module, const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_define_singleton_method(VALUE object, const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_define_module_function(VALUE module, const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_define_global_function(const char *name, VALUE (*func)(ANYARGS), int arity);
void rb_undef_method(VALUE module, const char *name);
void rb_undef(VALUE module, ID name);
const char *rb_class2name(VALUE module);
VALUE rb_mod_ancestors(VALUE module);
VALUE rb_mod_name(VALUE module);
VALUE rb_class_name(VALUE module);
VALUE rb_class_path(VALUE module);
VALUE rb_path2class(const char *path);
VALUE rb_path_to_class(VALUE path);
VALUE rb_class_instance_methods(int count, const VALUE *arguments, VALUE module);
VALUE rb_class_public_instance_methods(int count, const VALUE *arguments, VALUE module);
VALUE rb_class_protected_instance_methods(int count, const VALUE *arguments, VALUE module);
VALUE rb_class_private_instance_methods(int count, const VALUE *arguments, VALUE module);
VALUE rb_class_new(VALUE superclass);
VALUE rb_class_real(VALUE klass);
VALUE rb_class_superclass(VALUE klass);
VALUE rb_class_get_superclass(VALUE klass);
VALUE rb_call_super(int count, const VALUE *arguments);
VALUE rb_cvar_defined(VALUE klass, ID name);
VALUE rb_cvar_get(VALUE klass, ID name);
void rb_cvar_set(VALUE klass, ID name, VALUE value);
VALUE rb_cv_get(VALUE klass, const char *name);
void rb_cv_set(VALUE klass, const char *name, VALUE value);
void rb_define_class_variable(VALUE klass, const char *name, VALUE value);
void rb_define_attr(VALUE klass, const char *name, int read, int write);
void rb_include_module(VALUE klass, VALUE module);

enum ruby_value_type {
  RUBY_T_NONE = 0x00,
  RUBY_T_OBJECT = 0x01,
  RUBY_T_CLASS = 0x02,
  RUBY_T_MODULE = 0x03,
  RUBY_T_FLOAT = 0x04,
  RUBY_T_STRING = 0x05,
  RUBY_T_REGEXP = 0x06,
  RUBY_T_ARRAY = 0x07,
  RUBY_T_HASH = 0x08,
  RUBY_T_STRUCT = 0x09,
  RUBY_T_BIGNUM = 0x0a,
  RUBY_T_FILE = 0x0b,
  RUBY_T_DATA = 0x0c,
  RUBY_T_MATCH = 0x0d,
  RUBY_T_COMPLEX = 0x0e,
  RUBY_T_RATIONAL = 0x0f,
  RUBY_T_NIL = 0x11,
  RUBY_T_TRUE = 0x12,
  RUBY_T_FALSE = 0x13,
  RUBY_T_SYMBOL = 0x14,
  RUBY_T_FIXNUM = 0x15,
  RUBY_T_UNDEF = 0x16
};
#define T_NONE RUBY_T_NONE
#define T_OBJECT RUBY_T_OBJECT
#define T_CLASS RUBY_T_CLASS
#define T_MODULE RUBY_T_MODULE
#define T_FLOAT RUBY_T_FLOAT
#define T_STRING RUBY_T_STRING
#define T_REGEXP RUBY_T_REGEXP
#define T_ARRAY RUBY_T_ARRAY
#define T_HASH RUBY_T_HASH
#define T_STRUCT RUBY_T_STRUCT
#define T_BIGNUM RUBY_T_BIGNUM
#define T_FILE RUBY_T_FILE
#define T_DATA RUBY_T_DATA
#define T_MATCH RUBY_T_MATCH
#define T_COMPLEX RUBY_T_COMPLEX
#define T_RATIONAL RUBY_T_RATIONAL
#define T_NIL RUBY_T_NIL
#define T_TRUE RUBY_T_TRUE
#define T_FALSE RUBY_T_FALSE
#define T_SYMBOL RUBY_T_SYMBOL
#define T_FIXNUM RUBY_T_FIXNUM
#define T_UNDEF RUBY_T_UNDEF
int rb_type(VALUE object);
#define TYPE(v) rb_type((VALUE)(v))
static inline int rb_type_p(VALUE object, int type) { return rb_type(object) == type; }
#define RB_TYPE_P(v, t) rb_type_p((VALUE)(v), (t))
void rb_check_type(VALUE object, int type);
#define Check_Type(v, t) rb_check_type((VALUE)(v), (t))

typedef void (*RUBY_DATA_FUNC)(void *);
typedef struct rb_data_type_struct rb_data_type_t;
struct rb_data_type_struct {
  const char *wrap_struct_name;
  struct {
    RUBY_DATA_FUNC dmark;
    RUBY_DATA_FUNC dfree;
    size_t (*dsize)(const void *);
    RUBY_DATA_FUNC dcompact;
    void *reserved[1];
  } function;
  const rb_data_type_t *parent;
  void *data;
  VALUE flags;
};
#define RUBY_TYPED_FREE_IMMEDIATELY 1
#define RUBY_TYPED_EMBEDDABLE 2
#define RUBY_TYPED_WB_PROTECTED (1 << 5)
#define RUBY_TYPED_FROZEN_SHAREABLE (1 << 8)
#define RUBY_DEFAULT_FREE ((RUBY_DATA_FUNC)-1)
#define RUBY_NEVER_FREE ((RUBY_DATA_FUNC)0)
#define RUBY_TYPED_DEFAULT_FREE RUBY_DEFAULT_FREE
#define RUBY_TYPED_NEVER_FREE RUBY_NEVER_FREE

struct RBasic {
  VALUE flags;
  VALUE klass;
};
struct RBasic *rb_metorex_rbasic(VALUE object);
#define RBASIC(v) rb_metorex_rbasic((VALUE)(v))
#define RB_SPECIAL_CONST_P(v) (FIXNUM_P(v) || (VALUE)(v) < (VALUE)0x40)
#define SPECIAL_CONST_P(v) RB_SPECIAL_CONST_P(v)
#define RUBY_FL_SHAREABLE ((VALUE)1 << 8)
#define RUBY_FL_FREEZE ((VALUE)1 << 11)
#define RUBY_FL_USHIFT 12
#define RUBY_FL_USER0 ((VALUE)1 << RUBY_FL_USHIFT)
#define FL_SHAREABLE RUBY_FL_SHAREABLE
#define FL_FREEZE RUBY_FL_FREEZE
#define FL_USHIFT RUBY_FL_USHIFT
#define FL_USER0 RUBY_FL_USER0
#define RB_FL_TEST(v, f) (RB_SPECIAL_CONST_P(v) ? (VALUE)0 : (RBASIC(v)->flags & (VALUE)(f)))
#define RB_FL_SET(v, f) (RB_SPECIAL_CONST_P(v) ? (void)0 : (void)(RBASIC(v)->flags |= (VALUE)(f)))
#define RB_FL_UNSET(v, f) (RB_SPECIAL_CONST_P(v) ? (void)0 : (void)(RBASIC(v)->flags &= ~(VALUE)(f)))
#define FL_TEST(v, f) RB_FL_TEST(v, f)
#define FL_SET(v, f) RB_FL_SET(v, f)
#define FL_UNSET(v, f) RB_FL_UNSET(v, f)
struct RData {
  struct RBasic basic;
  RUBY_DATA_FUNC dmark;
  RUBY_DATA_FUNC dfree;
  void *data;
};
struct RTypedData {
  struct RBasic basic;
  VALUE fields_obj;
  const VALUE type;
  void *data;
};
void *rb_metorex_data_struct(VALUE object);
int rb_metorex_typeddata_p(VALUE object);
#define RDATA(v) ((struct RData *)rb_metorex_data_struct((VALUE)(v)))
#define RTYPEDDATA(v) ((struct RTypedData *)rb_metorex_data_struct((VALUE)(v)))
#define DATA_PTR(v) (RDATA(v)->data)
#define RTYPEDDATA_DATA(v) (RTYPEDDATA(v)->data)
#define RTYPEDDATA_GET_DATA(v) RTYPEDDATA_DATA(v)
#define RTYPEDDATA_P(v) rb_metorex_typeddata_p((VALUE)(v))
#define RTYPEDDATA_TYPE(v) ((const rb_data_type_t *)RTYPEDDATA(v)->type)

VALUE rb_data_object_wrap(VALUE klass, void *data, RUBY_DATA_FUNC mark, RUBY_DATA_FUNC free);
VALUE rb_data_object_zalloc(VALUE klass, size_t size, RUBY_DATA_FUNC mark, RUBY_DATA_FUNC free);
VALUE rb_data_typed_object_wrap(VALUE klass, void *data, const rb_data_type_t *type);
VALUE rb_data_typed_object_zalloc(VALUE klass, size_t size, const rb_data_type_t *type);
void *rb_check_typeddata(VALUE object, const rb_data_type_t *type);
int rb_typeddata_inherited_p(const rb_data_type_t *child, const rb_data_type_t *parent);
int rb_typeddata_is_kind_of(VALUE object, const rb_data_type_t *type);
#define Data_Wrap_Struct(klass, mark, free, data) \
  rb_data_object_wrap((klass), (data), (RUBY_DATA_FUNC)(mark), (RUBY_DATA_FUNC)(free))
#define Data_Make_Struct(klass, type, mark, free, sval) \
  rb_data_object_make((klass), (RUBY_DATA_FUNC)(mark), (RUBY_DATA_FUNC)(free), (void **)&(sval), sizeof(type))
#define Data_Get_Struct(obj, type, sval) ((sval) = (type *)DATA_PTR(obj))
#define TypedData_Wrap_Struct(klass, data_type, data) rb_data_typed_object_wrap((klass), (data), (data_type))
#define TypedData_Make_Struct(klass, type, data_type, sval) \
  rb_data_typed_object_make((klass), (data_type), (void **)&(sval), sizeof(type))
#define TypedData_Get_Struct(obj, type, data_type, sval) \
  ((sval) = (type *)rb_check_typeddata((obj), (data_type)))
static inline VALUE rb_data_object_make(VALUE klass, RUBY_DATA_FUNC mark, RUBY_DATA_FUNC free, void **data, size_t size) {
  VALUE made = rb_data_object_zalloc(klass, size, mark, free);
  *data = DATA_PTR(made);
  return made;
}
static inline VALUE rb_data_typed_object_make(VALUE klass, const rb_data_type_t *type, void **data, size_t size) {
  VALUE made = rb_data_typed_object_zalloc(klass, size, type);
  *data = RTYPEDDATA_DATA(made);
  return made;
}

VALUE rb_thread_current(void);
int rb_thread_alone(void);
VALUE rb_thread_local_aref(VALUE thread, ID name);
VALUE rb_thread_local_aset(VALUE thread, ID name, VALUE value);
VALUE rb_thread_wakeup(VALUE thread);
VALUE rb_thread_create(VALUE (*function)(void *), void *data);
void rb_thread_wait_for(struct timeval interval);
int ruby_native_thread_p(void);

typedef VALUE (*rb_alloc_func_t)(VALUE klass);
void rb_define_alloc_func(VALUE klass, rb_alloc_func_t allocator);
rb_alloc_func_t rb_get_alloc_func(VALUE klass);
void rb_undef_alloc_func(VALUE klass);
VALUE rb_obj_alloc(VALUE klass);
VALUE rb_obj_dup(VALUE object);
void rb_obj_call_init(VALUE object, int count, const VALUE *values);
VALUE rb_obj_class(VALUE object);
const char *rb_obj_classname(VALUE object);
VALUE rb_obj_freeze(VALUE object);
VALUE rb_obj_frozen_p(VALUE object);
void rb_check_frozen(VALUE object);
VALUE rb_obj_id(VALUE object);
VALUE rb_obj_is_instance_of(VALUE object, VALUE klass);
VALUE rb_obj_is_kind_of(VALUE object, VALUE klass);
VALUE rb_obj_method(VALUE object, VALUE name);
int rb_obj_method_arity(VALUE object, ID name);
int rb_respond_to(VALUE object, ID name);
int rb_obj_respond_to(VALUE object, ID name, int include_private);
int rb_method_boundp(VALUE klass, ID name, int exclude_private);
VALUE rb_special_const_p(VALUE object);
ID rb_to_id(VALUE name);
VALUE rb_to_int(VALUE object);
VALUE rb_check_to_integer(VALUE object, const char *method);
VALUE rb_convert_type(VALUE object, int type, const char *type_name, const char *method);
VALUE rb_check_convert_type(VALUE object, int type, const char *type_name, const char *method);
VALUE rb_check_array_type(VALUE object);
VALUE rb_check_string_type(VALUE object);
void rb_extend_object(VALUE object, VALUE module);
VALUE rb_obj_instance_eval(int count, const VALUE *values, VALUE object);
VALUE rb_any_to_s(VALUE object);
VALUE rb_equal(VALUE first, VALUE second);
VALUE rb_class_inherited_p(VALUE module, VALUE other);
VALUE rb_require(const char *feature);
VALUE rb_f_notimplement(int count, const VALUE *values, VALUE object, VALUE marker);
VALUE rb_attr_get(VALUE object, ID name);
VALUE rb_obj_instance_variables(VALUE object);
VALUE rb_iv_get(VALUE object, const char *name);
VALUE rb_iv_set(VALUE object, const char *name, VALUE value);
VALUE rb_ivar_get(VALUE object, ID name);
VALUE rb_ivar_set(VALUE object, ID name, VALUE value);
VALUE rb_ivar_defined(VALUE object, ID name);
size_t rb_ivar_count(VALUE object);
void rb_ivar_foreach(VALUE object, int (*function)(ID name, VALUE value, VALUE data), VALUE data);
void rb_copy_generic_ivar(VALUE clone, VALUE object);
void rb_free_generic_ivar(VALUE object);
#define BUILTIN_TYPE(v) rb_type((VALUE)(v))
#define FL_ABLE(v) (!RB_SPECIAL_CONST_P(v))
#define RB_OBJ_FROZEN(v) (RB_SPECIAL_CONST_P(v) || RB_FL_TEST((v), RUBY_FL_FREEZE))
#define OBJ_FROZEN(v) RB_OBJ_FROZEN(v)
VALUE rb_marshal_dump(VALUE object, VALUE port);
VALUE rb_marshal_load(VALUE port);
VALUE rb_define_finalizer(VALUE object, VALUE finalizer);
VALUE rb_undefine_finalizer(VALUE object);

char *rb_rstring_ptr(VALUE string);
long rb_rstring_len(VALUE string);
#define RSTRING_PTR(v) rb_rstring_ptr((VALUE)(v))
#define RSTRING_END(v) (RSTRING_PTR(v) + RSTRING_LEN(v))
#define RSTRING_LEN(v) rb_rstring_len((VALUE)(v))
#define RSTRING_LENINT(v) ((int)RSTRING_LEN(v))
#define SafeStringValue(v) StringValue(v)
#define ST2FIX(h) LONG2FIX((long)(h))
#define PRINTF_ARGS(declaration, format_index, first_index) declaration

VALUE rb_String(VALUE object);
VALUE rb_str_to_str(VALUE object);
VALUE rb_str_dup(VALUE string);
VALUE rb_str_new_shared(VALUE string);
VALUE rb_str_new_frozen(VALUE string);
VALUE rb_str_new_with_class(VALUE string, const char *text, long length);
#define rb_str_new3 rb_str_new_shared
#define rb_str_new4 rb_str_new_frozen
#define rb_str_new5 rb_str_new_with_class
VALUE rb_str_buf_new(long capacity);
VALUE rb_str_buf_new_cstr(const char *text);
#define rb_str_buf_new2 rb_str_buf_new_cstr
VALUE rb_str_tmp_new(long length);
VALUE rb_obj_reveal(VALUE object, VALUE klass);
size_t rb_str_capacity(VALUE string);
void rb_str_modify(VALUE string);
void rb_str_modify_expand(VALUE string, long expand);
void rb_str_set_len(VALUE string, long length);
VALUE rb_str_resize(VALUE string, long length);
VALUE rb_str_drop_bytes(VALUE string, long length);
void rb_str_free(VALUE string);
VALUE rb_str_locktmp(VALUE string);
VALUE rb_str_unlocktmp(VALUE string);
VALUE rb_usascii_str_new(const char *text, long length);
VALUE rb_usascii_str_new_cstr(const char *text);
VALUE rb_usascii_str_new_static(const char *text, long length);
#define rb_usascii_str_new_lit(text) rb_usascii_str_new_static((text), (long)(sizeof(text) - 1))
VALUE rb_utf8_str_new(const char *text, long length);
VALUE rb_utf8_str_new_cstr(const char *text);
VALUE rb_utf8_str_new_static(const char *text, long length);
VALUE rb_external_str_new(const char *text, long length);
VALUE rb_external_str_new_cstr(const char *text);
VALUE rb_locale_str_new(const char *text, long length);
VALUE rb_locale_str_new_cstr(const char *text);
VALUE rb_interned_str(const char *text, long length);
VALUE rb_interned_str_cstr(const char *text);
VALUE rb_str_to_interned_str(VALUE string);
VALUE rb_str_plus(VALUE first, VALUE second);
VALUE rb_str_times(VALUE string, VALUE count);
VALUE rb_str_buf_append(VALUE string, VALUE added);
VALUE rb_str_buf_cat(VALUE string, const char *text, long length);
VALUE rb_str_cat(VALUE string, const char *text, long length);
VALUE rb_str_cat_cstr(VALUE string, const char *text);
#define rb_str_cat2 rb_str_cat_cstr
int rb_str_cmp(VALUE first, VALUE second);
VALUE rb_str_equal(VALUE first, VALUE second);
VALUE rb_str_length(VALUE string);
long rb_str_strlen(VALUE string);
long rb_str_sublen(VALUE string, long byte_offset);
char *rb_str_subpos(VALUE string, long start, long *length);
VALUE rb_str_subseq(VALUE string, long start, long length);
VALUE rb_str_substr(VALUE string, long start, long length);
void rb_str_update(VALUE string, long start, long length, VALUE replacement);
VALUE rb_str_split(VALUE string, const char *separator);
VALUE rb_str_inspect(VALUE string);
VALUE rb_str_intern(VALUE string);
VALUE rb_str_freeze(VALUE string);
st_index_t rb_str_hash(VALUE string);
VALUE rb_str2inum(VALUE string, int base);
VALUE rb_cstr2inum(const char *text, int base);
VALUE rb_cstr_to_inum(const char *text, int base, int strict);
VALUE rb_str_encode(VALUE string, VALUE encoding, int flags, VALUE options);
VALUE rb_str_export(VALUE string);
VALUE rb_str_export_locale(VALUE string);

long rb_rarray_len(VALUE array);
const VALUE *rb_rarray_ptr(VALUE array);
#define RARRAY_LEN(v) rb_rarray_len((VALUE)(v))
#define RARRAY_PTR(v) ((VALUE *)rb_rarray_ptr((VALUE)(v)))
VALUE rb_ary_new_from_values(long count, const VALUE *values);
VALUE rb_ary_entry(VALUE array, long offset);
#define RARRAY_LENINT(v) ((int)RARRAY_LEN(v))
VALUE rb_ary_new(void);
VALUE rb_ary_new_capa(long capacity);
#define rb_ary_new2 rb_ary_new_capa
VALUE rb_ary_push(VALUE array, VALUE element);
VALUE rb_ary_pop(VALUE array);
void rb_ary_store(VALUE array, long index, VALUE element);
VALUE rb_ary_dup(VALUE array);
#define RARRAY_AREF(v, i) rb_ary_entry((VALUE)(v), (long)(i))
#define RARRAY_ASET(v, i, value) rb_ary_store((VALUE)(v), (long)(i), (VALUE)(value))
#define rb_ary_new3 rb_ary_new_from_args
#define rb_ary_new4 rb_ary_new_from_values
VALUE rb_Array(VALUE object);
VALUE rb_ary_aref(int count, const VALUE *values, VALUE array);
VALUE rb_ary_cat(VALUE array, const VALUE *values, long count);
VALUE rb_ary_clear(VALUE array);
VALUE rb_ary_concat(VALUE array, VALUE other);
VALUE rb_ary_delete(VALUE array, VALUE element);
VALUE rb_ary_delete_at(VALUE array, long index);
VALUE rb_ary_freeze(VALUE array);
VALUE rb_ary_includes(VALUE array, VALUE element);
VALUE rb_ary_join(VALUE array, VALUE separator);
VALUE rb_ary_plus(VALUE array, VALUE other);
VALUE rb_ary_reverse(VALUE array);
VALUE rb_ary_rotate(VALUE array, long count);
VALUE rb_ary_shift(VALUE array);
VALUE rb_ary_sort(VALUE array);
VALUE rb_ary_sort_bang(VALUE array);
VALUE rb_ary_subseq(VALUE array, long start, long length);
VALUE rb_ary_to_ary(VALUE object);
VALUE rb_ary_to_s(VALUE array);
VALUE rb_assoc_new(VALUE first, VALUE second);
void rb_mem_clear(VALUE *values, long count);

typedef VALUE rb_enumerator_size_func(VALUE object, VALUE arguments, VALUE enumerator);
VALUE rb_enumeratorize(VALUE object, VALUE method, int count, const VALUE *arguments);
VALUE rb_enumeratorize_with_size(VALUE object, VALUE method, int count, const VALUE *arguments,
                                 rb_enumerator_size_func *size_function);

#define INTEGER_PACK_MSWORD_FIRST 0x01
#define INTEGER_PACK_LSWORD_FIRST 0x02
#define INTEGER_PACK_MSBYTE_FIRST 0x10
#define INTEGER_PACK_LSBYTE_FIRST 0x20
#define INTEGER_PACK_NATIVE_BYTE_ORDER 0x40
#define INTEGER_PACK_2COMP 0x80
#define INTEGER_PACK_FORCE_BIGNUM 0x100
#define INTEGER_PACK_NEGATIVE 0x200
#define INTEGER_PACK_FORCE_GENERIC_IMPLEMENTATION 0x400
#define INTEGER_PACK_LITTLE_ENDIAN (INTEGER_PACK_LSWORD_FIRST | INTEGER_PACK_LSBYTE_FIRST)
#define INTEGER_PACK_BIG_ENDIAN (INTEGER_PACK_MSWORD_FIRST | INTEGER_PACK_MSBYTE_FIRST)
int rb_integer_pack(VALUE value, void *words, size_t word_count, size_t word_size, size_t nails,
                    int flags);

VALUE rb_float_new(double number);
double rb_float_value(VALUE float_value);
int rb_float_type_p(VALUE value);
#define DBL2NUM(d) rb_float_new(d)
#define RFLOAT_VALUE(v) rb_float_value((VALUE)(v))
#define RB_FLOAT_TYPE_P(v) rb_float_type_p((VALUE)(v))
VALUE rb_Float(VALUE value);

VALUE rb_Complex(VALUE real, VALUE imaginary);
VALUE rb_complex_new(VALUE real, VALUE imaginary);
#define rb_Complex1(x) rb_Complex((x), INT2FIX(0))
#define rb_Complex2(x, y) rb_Complex((x), (y))
#define rb_complex_new1(x) rb_complex_new((x), INT2FIX(0))
#define rb_complex_new2(x, y) rb_complex_new((x), (y))

VALUE rb_Rational(VALUE numerator, VALUE denominator);
VALUE rb_rational_new(VALUE numerator, VALUE denominator);
VALUE rb_rational_num(VALUE rational);
VALUE rb_rational_den(VALUE rational);
#define rb_Rational1(x) rb_Rational((x), INT2FIX(1))
#define rb_Rational2(x, y) rb_Rational((x), (y))
#define rb_rational_new1(x) rb_rational_new((x), INT2FIX(1))
#define rb_rational_new2(x, y) rb_rational_new((x), (y))

#define RB_BLOCK_CALL_FUNC_ARGLIST(yielded_arg, callback_arg) \
  VALUE yielded_arg, VALUE callback_arg, int argc, const VALUE *argv, VALUE blockarg
typedef VALUE rb_block_call_func(RB_BLOCK_CALL_FUNC_ARGLIST(yielded_arg, callback_arg));
typedef rb_block_call_func *rb_block_call_func_t;
VALUE rb_block_call(VALUE receiver, ID name, int count, const VALUE *values, rb_block_call_func_t function, VALUE data);
VALUE rb_block_lambda(void);
void rb_need_block(void);
ID rb_frame_this_func(void);
VALUE rb_protect(VALUE (*function)(VALUE), VALUE data, int *state);
NORETURN(void rb_jump_tag(int state));
VALUE rb_rescue(VALUE (*body)(VALUE), VALUE data, VALUE (*rescue)(VALUE, VALUE), VALUE rescue_data);
VALUE rb_metorex_rescue2(VALUE (*body)(VALUE), VALUE data, VALUE (*rescue)(VALUE, VALUE), VALUE rescue_data,
                         int count, const VALUE *classes);
static inline VALUE rb_rescue2(VALUE (*body)(VALUE), VALUE data, VALUE (*rescue)(VALUE, VALUE), VALUE rescue_data,
                               ...) {
  VALUE classes[64];
  int count = 0;
  va_list held;
  va_start(held, rescue_data);
  for (VALUE klass = va_arg(held, VALUE); klass != 0 && count < 64; klass = va_arg(held, VALUE)) {
    classes[count++] = klass;
  }
  va_end(held);
  return rb_metorex_rescue2(body, data, rescue, rescue_data, count, classes);
}
VALUE rb_ensure(VALUE (*body)(VALUE), VALUE data, VALUE (*ensure)(VALUE), VALUE ensure_data);
VALUE rb_catch(const char *tag, rb_block_call_func_t function, VALUE data);
VALUE rb_catch_obj(VALUE tag, rb_block_call_func_t function, VALUE data);
NORETURN(void rb_throw(const char *tag, VALUE value));
NORETURN(void rb_throw_obj(VALUE tag, VALUE value));
VALUE rb_eval_string(const char *source);
VALUE rb_eval_string_protect(const char *source, int *state);
VALUE rb_exec_recursive(VALUE (*function)(VALUE, VALUE, int), VALUE object, VALUE data);
void rb_set_end_proc(void (*function)(VALUE), VALUE data);
VALUE rb_f_sprintf(int count, const VALUE *values);
VALUE rb_str_format(int count, const VALUE *values, VALUE format);
VALUE rb_make_backtrace(void);
VALUE rb_funcallv_kw(VALUE receiver, ID name, int count, const VALUE *arguments, int keywords);
VALUE rb_funcallv_public(VALUE receiver, ID name, int count, const VALUE *arguments);
VALUE rb_funcall_with_block(VALUE receiver, ID name, int count, const VALUE *arguments, VALUE block);
VALUE rb_funcall_with_block_kw(VALUE receiver, ID name, int count, const VALUE *arguments, VALUE block,
                               int keywords);
VALUE rb_check_funcall(VALUE receiver, ID name, int count, const VALUE *arguments);
NORETURN(void rb_sys_fail(const char *message));
NORETURN(void rb_syserr_fail(int number, const char *message));
NORETURN(void rb_syserr_fail_str(int number, VALUE message));

VALUE rb_fiber_new(rb_block_call_func_t function, VALUE data);
VALUE rb_fiber_current(void);
VALUE rb_fiber_alive_p(VALUE fiber);
VALUE rb_fiber_resume(VALUE fiber, int count, const VALUE *values);
VALUE rb_fiber_yield(int count, const VALUE *values);
VALUE rb_fiber_transfer(VALUE fiber, int count, const VALUE *values);
VALUE rb_fiber_raise(VALUE fiber, int count, VALUE *values);

VALUE rb_string_value(volatile VALUE *pointer);
char *rb_string_value_ptr(volatile VALUE *pointer);
char *rb_string_value_cstr(volatile VALUE *pointer);
#define StringValue(v) rb_string_value(&(v))
#define StringValuePtr(v) rb_string_value_ptr(&(v))
#define StringValueCStr(v) rb_string_value_cstr(&(v))

VALUE rb_reg_new(const char *source, long length, int options);
VALUE rb_reg_regcomp(VALUE source);
int rb_reg_options(VALUE regexp);
VALUE rb_reg_match(VALUE regexp, VALUE string);
VALUE rb_reg_nth_match(int nth, VALUE match);
VALUE rb_backref_get(void);
void rb_backref_set(VALUE match);
int rb_memcicmp(const void *first, const void *second, long length);

VALUE rb_mutex_new(void);
VALUE rb_mutex_locked_p(VALUE mutex);
VALUE rb_mutex_trylock(VALUE mutex);
VALUE rb_mutex_lock(VALUE mutex);
VALUE rb_mutex_unlock(VALUE mutex);
VALUE rb_mutex_sleep(VALUE mutex, VALUE timeout);
VALUE rb_mutex_synchronize(VALUE mutex, VALUE (*function)(VALUE data), VALUE data);

#define TIMET2NUM(v) LONG2NUM(v)
#define NUM2TIMET(v) NUM2LONG(v)
VALUE rb_time_new(time_t seconds, long microseconds);
VALUE rb_time_nano_new(time_t seconds, long nanoseconds);
VALUE rb_time_num_new(VALUE seconds, VALUE offset);
VALUE rb_time_timespec_new(const struct timespec *time, int offset);
void rb_timespec_now(struct timespec *time);
struct timeval rb_time_interval(VALUE seconds);
struct timeval rb_time_timeval(VALUE time);
struct timespec rb_time_timespec(VALUE time);

VALUE rb_errinfo(void);
void rb_set_errinfo(VALUE exception);
VALUE rb_exc_new(VALUE klass, const char *text, long length);
VALUE rb_exc_new_cstr(VALUE klass, const char *text);
VALUE rb_exc_new_str(VALUE klass, VALUE message);
#define rb_exc_new2 rb_exc_new_cstr
#define rb_exc_new3 rb_exc_new_str
NORETURN(void rb_exc_raise(VALUE exception));
NORETURN(void rb_error_frozen_object(VALUE object));
VALUE rb_syserr_new(int number, const char *message);
VALUE rb_syserr_new_str(int number, VALUE message);
VALUE rb_make_exception(int count, const VALUE *values);

VALUE rb_range_new(VALUE first, VALUE last, int exclusive);
int rb_range_values(VALUE range, VALUE *first, VALUE *last, int *exclusive);
VALUE rb_range_beg_len(VALUE range, long *start, long *length, long sequence_length, int err);
typedef struct {
  VALUE begin;
  VALUE end;
  VALUE step;
  int exclude_end;
} rb_arithmetic_sequence_components_t;
int rb_arithmetic_sequence_extract(VALUE sequence, rb_arithmetic_sequence_components_t *parts);
VALUE rb_arithmetic_sequence_beg_len_step(VALUE sequence, long *start, long *length, long *step,
                                          long sequence_length, int err);

VALUE rb_str_new(const char *text, long length);
VALUE rb_str_new_cstr(const char *text);
#define rb_str_new2 rb_str_new_cstr
VALUE rb_str_append(VALUE string, VALUE added);
VALUE rb_obj_as_string(VALUE object);
VALUE rb_inspect(VALUE object);
void rb_metorex_adopt_encoding(VALUE string, VALUE source);
void rb_warn_message(VALUE message, int verbose_only);
typedef enum {
  RB_WARN_CATEGORY_NONE,
  RB_WARN_CATEGORY_DEPRECATED,
  RB_WARN_CATEGORY_EXPERIMENTAL,
  RB_WARN_CATEGORY_PERFORMANCE
} rb_warning_category_t;
void rb_metorex_category_warn(rb_warning_category_t category, VALUE message);

VALUE rb_class_new_instance(int count, const VALUE *values, VALUE klass);
VALUE rb_class_new_instance_kw(int count, const VALUE *values, VALUE klass, int keywords);
VALUE rb_metorex_struct_define(const char *name, int count, const char **names);
VALUE rb_metorex_struct_define_under(VALUE outer, const char *name, int count, const char **names);
VALUE rb_metorex_data_define(VALUE superclass, int count, const char **names);
VALUE rb_struct_s_members(VALUE klass);
VALUE rb_struct_members(VALUE instance);
VALUE rb_struct_size(VALUE instance);
VALUE rb_struct_aref(VALUE instance, VALUE key);
VALUE rb_struct_aset(VALUE instance, VALUE key, VALUE value);
VALUE rb_struct_getmember(VALUE instance, ID name);
VALUE rb_struct_initialize(VALUE instance, VALUE values);

/* The member names after `last`, up to the NULL that ends them. */
#define METOREX_GATHER_NAMES(last)                       \
  const char *names[64];                                 \
  int count = 0;                                         \
  va_list held;                                          \
  va_start(held, last);                                  \
  for (const char *name = va_arg(held, const char *);    \
       name != NULL && count < 64;                       \
       name = va_arg(held, const char *)) {              \
    names[count++] = name;                               \
  }                                                      \
  va_end(held)

static inline VALUE rb_struct_define(const char *name, ...) {
  METOREX_GATHER_NAMES(name);
  return rb_metorex_struct_define(name, count, names);
}

static inline VALUE rb_struct_define_under(VALUE outer, const char *name, ...) {
  METOREX_GATHER_NAMES(name);
  return rb_metorex_struct_define_under(outer, name, count, names);
}

static inline VALUE rb_data_define(VALUE superclass, ...) {
  METOREX_GATHER_NAMES(superclass);
  return rb_metorex_data_define(superclass, count, names);
}

/* One value for each member of `klass`, in order. */
static inline VALUE rb_struct_new(VALUE klass, ...) {
  long count = NUM2LONG(rb_funcallv(rb_struct_s_members(klass), rb_intern("size"), 0, NULL));
  VALUE values[count > 0 ? count : 1];
  va_list held;
  va_start(held, klass);
  for (long index = 0; index < count; index++) {
    values[index] = va_arg(held, VALUE);
  }
  va_end(held);
  return rb_class_new_instance((int)count, values, klass);
}

long rb_big2long(VALUE value);
long long rb_big2ll(VALUE value);
unsigned long rb_big2ulong(VALUE value);
double rb_big2dbl(VALUE value);
VALUE rb_dbl2big(double number);
VALUE rb_big2str(VALUE value, int base);
int rb_big_sign(VALUE value);
#define RBIGNUM_SIGN(v) rb_big_sign((VALUE)(v))
#define RBIGNUM_POSITIVE_P(v) (RBIGNUM_SIGN(v) == 1)
#define RBIGNUM_NEGATIVE_P(v) (RBIGNUM_SIGN(v) == 0)
VALUE rb_big_cmp(VALUE first, VALUE second);
void rb_big_pack(VALUE value, unsigned long *longs, long count);
size_t rb_absint_size(VALUE value, int *unused_bits);
double rb_num2dbl(VALUE value);
#define NUM2DBL(v) rb_num2dbl((VALUE)(v))

NORETURN(void rb_cmperr(VALUE first, VALUE second));
NORETURN(void rb_num_zerodiv(void));
int rb_cmpint(VALUE answer, VALUE first, VALUE second);
VALUE rb_num_coerce_bin(VALUE first, VALUE second, ID name);
VALUE rb_num_coerce_cmp(VALUE first, VALUE second, ID name);
VALUE rb_num_coerce_relop(VALUE first, VALUE second, ID name);
VALUE rb_Integer(VALUE value);
short rb_num2short(VALUE value);
#define NUM2SHORT(v) rb_num2short((VALUE)(v))
char rb_num2char(VALUE value);
#define NUM2CHR(v) rb_num2char((VALUE)(v))
#define CHR2FIX(c) INT2FIX((long)((c) & 0xff))
int rb_absint_singlebit_p(VALUE value);

VALUE rb_proc_new(rb_block_call_func_t function, VALUE data);
int rb_proc_arity(VALUE procedure);
VALUE rb_obj_is_proc(VALUE value);
VALUE rb_proc_call(VALUE procedure, VALUE values);
VALUE rb_proc_call_kw(VALUE procedure, VALUE values, int keywords);
VALUE rb_proc_call_with_block(VALUE procedure, int count, const VALUE *values, VALUE block);
VALUE rb_proc_call_with_block_kw(VALUE procedure, int count, const VALUE *values, VALUE block, int keywords);

void rb_gc_register_address(VALUE *address);
void rb_gc_mark(VALUE object);
void rb_gc_mark_movable(VALUE object);
void rb_gc_mark_maybe(VALUE object);
void rb_gc_mark_locations(const VALUE *start, const VALUE *end);
VALUE rb_gc_location(VALUE object);
void rb_gc_unregister_address(VALUE *address);
void rb_global_variable(VALUE *address);
void rb_gc_register_mark_object(VALUE object);
void rb_gc_adjust_memory_usage(ssize_t difference);
VALUE rb_gc_enable(void);
VALUE rb_gc_disable(void);
void rb_gc(void);
VALUE rb_gc_start(void);
size_t rb_gc_count(void);
VALUE rb_gc_latest_gc_info(VALUE key_or_hash);
#define NUM2SSIZET(v) ((ssize_t)NUM2LONG(v))
#define NUM2SIZET(v) ((size_t)NUM2ULONG(v))

typedef VALUE rb_gvar_getter_t(ID id, VALUE *data);
typedef void rb_gvar_setter_t(VALUE value, ID id, VALUE *data);
void rb_define_hooked_variable(const char *name, VALUE *variable, rb_gvar_getter_t *getter, rb_gvar_setter_t *setter);
void rb_define_variable(const char *name, VALUE *variable);
void rb_define_readonly_variable(const char *name, const VALUE *variable);
void rb_define_virtual_variable(const char *name, rb_gvar_getter_t *getter, rb_gvar_setter_t *setter);
VALUE rb_gv_get(const char *name);
VALUE rb_gv_set(const char *name, VALUE value);
VALUE rb_f_global_variables(void);
VALUE rb_lastline_get(void);
void rb_lastline_set(VALUE line);
VALUE rb_metorex_default_rs(void);
/* These follow the Ruby globals they stand for, so each read reads the
 * global as it is now. */
#define rb_fs rb_gv_get("$;")
#define rb_rs rb_gv_get("$/")
#define rb_output_fs rb_gv_get("$,")
#define rb_output_rs rb_gv_get("$\\")
#define rb_stdin rb_gv_get("$stdin")
#define rb_stdout rb_gv_get("$stdout")
#define rb_stderr rb_gv_get("$stderr")
#define rb_defout rb_stdout
#define rb_default_rs rb_metorex_default_rs()

VALUE rb_get_path(VALUE object);
#define FilePathValue(v) ((v) = rb_get_path((VALUE)(v)))
VALUE rb_file_open(const char *name, const char *mode);
VALUE rb_file_open_str(VALUE name, const char *mode);

/* Variadic, so it gathers its arguments here and hands them to the
 * function metorex exports. */
static inline VALUE rb_funcall(VALUE receiver, ID name, int count, ...) {
  VALUE arguments[count > 0 ? count : 1];
  va_list held;
  va_start(held, count);
  for (int index = 0; index < count; index++) {
    arguments[index] = va_arg(held, VALUE);
  }
  va_end(held);
  return rb_funcallv(receiver, name, count, arguments);
}

int rb_block_given_p(void);
VALUE rb_yield(VALUE value);
VALUE rb_yield_values2(int count, const VALUE *values);
VALUE rb_yield_splat(VALUE values);

/* Variadic, so it gathers its arguments here and hands them to the
 * function metorex exports. */
static inline VALUE rb_yield_values(int count, ...) {
  VALUE values[count > 0 ? count : 1];
  va_list held;
  va_start(held, count);
  for (int index = 0; index < count; index++) {
    values[index] = va_arg(held, VALUE);
  }
  va_end(held);
  return rb_yield_values2(count, values);
}

VALUE rb_set_new(void);
VALUE rb_set_new_capa(size_t capacity);
bool rb_set_lookup(VALUE set, VALUE element);
bool rb_set_add(VALUE set, VALUE element);
bool rb_set_delete(VALUE set, VALUE element);
VALUE rb_set_clear(VALUE set);
size_t rb_set_size(VALUE set);
void rb_set_foreach(VALUE set, int (*function)(VALUE element, VALUE data), VALUE data);

#define UNLIMITED_ARGUMENTS (-1)
NORETURN(void rb_error_arity(int given, int min, int max));
NORETURN(void rb_scan_args_bad_format(const char *format));

int rb_keyword_given_p(void);
VALUE rb_block_proc(void);
VALUE rb_hash_dup(VALUE hash);
VALUE rb_hash(VALUE object);
VALUE rb_Hash(VALUE object);
VALUE rb_hash_new(void);
VALUE rb_hash_new_capa(long capacity);
VALUE rb_ident_hash_new(void);
VALUE rb_hash_freeze(VALUE hash);
VALUE rb_hash_aref(VALUE hash, VALUE key);
VALUE rb_hash_aset(VALUE hash, VALUE key, VALUE value);
VALUE rb_hash_clear(VALUE hash);
VALUE rb_hash_delete(VALUE hash, VALUE key);
VALUE rb_hash_delete_if(VALUE hash);
VALUE rb_hash_fetch(VALUE hash, VALUE key);
VALUE rb_hash_lookup(VALUE hash, VALUE key);
VALUE rb_hash_lookup2(VALUE hash, VALUE key, VALUE missing);
VALUE rb_hash_size(VALUE hash);
VALUE rb_hash_set_ifnone(VALUE hash, VALUE default_value);
void rb_hash_bulk_insert(long count, const VALUE *pairs, VALUE hash);
void rb_hash_foreach(VALUE hash, int (*function)(VALUE key, VALUE value, VALUE argument), VALUE argument);
st_index_t rb_hash_start(st_index_t hash);
st_index_t rb_hash_uint32(st_index_t hash, uint32_t word);
st_index_t rb_hash_uint(st_index_t hash, st_index_t word);
st_index_t rb_hash_end(st_index_t hash);

#define RB_SCAN_ARGS_PASS_CALLED_KEYWORDS 0
#define RB_SCAN_ARGS_KEYWORDS 1
#define RB_SCAN_ARGS_LAST_HASH_KEYWORDS 3
#define RB_NO_KEYWORDS 0
#define RB_PASS_KEYWORDS 1
#define RB_PASS_CALLED_KEYWORDS !!rb_keyword_given_p()

static inline int metorex_scan_args_digit(const char *cursor) { return *cursor >= '0' && *cursor <= '9'; }

/* Reads a format of leading, optional, splat and trailing counts, then ':'
 * for a keyword Hash and '&' for the block, such as "11*1:&", and stores
 * each argument through the pointer that follows the format, as MRI's
 * rb_scan_args_set does. `keyword_flag` says when the last argument is the
 * keyword Hash. */
static inline int metorex_scan_args(int keyword_flag, int count, const VALUE *arguments, const char *format,
                                    va_list *held) {
  const char *cursor = format;
  int leading = 0, optional = 0, trailing = 0, splat = 0, keywords = 0, block = 0;
  if (metorex_scan_args_digit(cursor)) {
    leading = *cursor++ - '0';
    if (metorex_scan_args_digit(cursor)) optional = *cursor++ - '0';
  }
  if (*cursor == '*') {
    splat = 1;
    cursor++;
  }
  if (metorex_scan_args_digit(cursor)) trailing = *cursor++ - '0';
  if (*cursor == ':') {
    keywords = 1;
    cursor++;
  }
  if (*cursor == '&') {
    block = 1;
    cursor++;
  }
  if (*cursor != '\0') rb_scan_args_bad_format(format);
  VALUE hash = Qnil;
  if (keywords && count > 0) {
    VALUE last = arguments[count - 1];
    int is_keywords = keyword_flag == RB_SCAN_ARGS_KEYWORDS ||
                      (keyword_flag == RB_SCAN_ARGS_PASS_CALLED_KEYWORDS && rb_keyword_given_p()) ||
                      (keyword_flag == RB_SCAN_ARGS_LAST_HASH_KEYWORDS &&
                       RTEST(rb_funcallv(last, rb_intern("is_a?"), 1, &rb_cHash)));
    if (is_keywords) {
      hash = rb_hash_dup(last);
      count--;
    }
  }
  int required = leading + trailing;
  if (count < required || (!splat && count > required + optional)) {
    rb_error_arity(count, required, splat ? UNLIMITED_ARGUMENTS : required + optional);
  }
  int index = 0;
  for (int slot = 0; slot < leading; slot++) {
    VALUE *target = va_arg(*held, VALUE *);
    if (target) *target = arguments[index];
    index++;
  }
  for (int slot = 0; slot < optional; slot++) {
    VALUE *target = va_arg(*held, VALUE *);
    if (index < count - trailing) {
      if (target) *target = arguments[index];
      index++;
    } else if (target) {
      *target = Qnil;
    }
  }
  if (splat) {
    int splat_count = count - index - trailing;
    VALUE *target = va_arg(*held, VALUE *);
    if (target) *target = rb_ary_new_from_values(splat_count > 0 ? splat_count : 0, arguments + index);
    if (splat_count > 0) index += splat_count;
  }
  for (int slot = 0; slot < trailing; slot++) {
    VALUE *target = va_arg(*held, VALUE *);
    if (target) *target = arguments[index];
    index++;
  }
  if (keywords) {
    VALUE *target = va_arg(*held, VALUE *);
    if (target) *target = hash;
  }
  if (block) {
    VALUE *target = va_arg(*held, VALUE *);
    if (target) *target = rb_block_given_p() ? rb_block_proc() : Qnil;
  }
  return count;
}

static inline int rb_scan_args(int count, const VALUE *arguments, const char *format, ...) {
  va_list held;
  va_start(held, format);
  int answered = metorex_scan_args(RB_SCAN_ARGS_PASS_CALLED_KEYWORDS, count, arguments, format, &held);
  va_end(held);
  return answered;
}

static inline int rb_scan_args_kw(int keyword_flag, int count, const VALUE *arguments, const char *format, ...) {
  va_list held;
  va_start(held, format);
  int answered = metorex_scan_args(keyword_flag, count, arguments, format, &held);
  va_end(held);
  return answered;
}

NORETURN(void rb_out_of_int(long number));
static inline int rb_long2int(long number) {
  if (number > INT_MAX || number < INT_MIN) rb_out_of_int(number);
  return (int)number;
}

/* The `count` VALUEs after `count`, as an Array. */
static inline VALUE rb_ary_new_from_args(long count, ...) {
  VALUE values[count > 0 ? count : 1];
  va_list held;
  va_start(held, count);
  for (long index = 0; index < count; index++) values[index] = va_arg(held, VALUE);
  va_end(held);
  return rb_ary_new_from_values(count, values);
}

int rb_get_kwargs(VALUE hash, const ID *names, int required, int optional, VALUE *values);
VALUE rb_ary_unshift(VALUE array, VALUE element);
NORETURN(void rb_iter_break(void));
NORETURN(void rb_iter_break_value(VALUE value));
const char *rb_sourcefile(void);
int rb_sourceline(void);

#include "ruby/internal/format.h"

#ifdef __cplusplus
}
#endif

#endif
