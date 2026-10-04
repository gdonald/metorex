#ifndef RUBY_ENCODING_H
#define RUBY_ENCODING_H 1

#include "ruby/ruby.h"

#ifdef __cplusplus
extern "C" {
#endif

/* metorex makes one of these for each encoding and keeps it for the life
 * of the program. */
typedef struct rb_encoding {
  const char *name;
  VALUE encoding;
} rb_encoding;

rb_encoding *rb_enc_get(VALUE object);
rb_encoding *rb_utf8_encoding(void);
rb_encoding *rb_usascii_encoding(void);
rb_encoding *rb_ascii8bit_encoding(void);
VALUE rb_enc_from_encoding(rb_encoding *encoding);
#define rb_enc_name(encoding) ((encoding)->name)
rb_encoding *rb_to_encoding(VALUE encoding);
#define ECONV_UNIVERSAL_NEWLINE_DECORATOR 0x00000100

#define ENC_CODERANGE_UNKNOWN 0
#define ENC_CODERANGE_7BIT 0x100000
#define ENC_CODERANGE_VALID 0x200000
#define ENC_CODERANGE_BROKEN 0x300000
#define RUBY_ENC_CODERANGE_UNKNOWN ENC_CODERANGE_UNKNOWN
#define RUBY_ENC_CODERANGE_7BIT ENC_CODERANGE_7BIT
#define RUBY_ENC_CODERANGE_VALID ENC_CODERANGE_VALID
#define RUBY_ENC_CODERANGE_BROKEN ENC_CODERANGE_BROKEN
int rb_enc_str_coderange(VALUE string);
int rb_enc_str_asciionly_p(VALUE string);
#define ENC_CODERANGE_ASCIIONLY(string) (rb_enc_str_coderange(string) == ENC_CODERANGE_7BIT)
/* metorex works a string's coderange out each time it is asked for, so
 * there is nothing held to clear. */
#define RB_ENC_CODERANGE_CLEAR(string) ((void)(string))
#define ENC_CODERANGE_CLEAR(string) RB_ENC_CODERANGE_CLEAR(string)

#define MBCLEN_CHARFOUND_P(length) (0 < (length))
#define MBCLEN_CHARFOUND_LEN(length) (length)
#define MBCLEN_INVALID_P(length) ((length) == -1)
#define MBCLEN_NEEDMORE_P(length) ((length) < -1)
#define MBCLEN_NEEDMORE_LEN(length) (-1 - (length))

int rb_enc_get_index(VALUE object);
void rb_enc_set_index(VALUE object, int index);
#define ENCODING_GET(object) rb_enc_get_index(object)
#define ENCODING_SET(object, index) rb_enc_set_index((object), (index))
VALUE rb_enc_associate(VALUE object, rb_encoding *encoding);
VALUE rb_enc_associate_index(VALUE object, int index);
void rb_enc_copy(VALUE destination, VALUE source);
VALUE rb_obj_encoding(VALUE object);
rb_encoding *rb_enc_compatible(VALUE first, VALUE second);
rb_encoding *rb_enc_check(VALUE first, VALUE second);

rb_encoding *rb_enc_find(const char *name);
int rb_enc_find_index(const char *name);
rb_encoding *rb_enc_from_index(int index);
int rb_enc_to_index(rb_encoding *encoding);
int rb_to_encoding_index(VALUE encoding);
int rb_enc_alias(const char *alias, const char *original);
int rb_define_dummy_encoding(const char *name);
rb_encoding *rb_locale_encoding(void);
rb_encoding *rb_filesystem_encoding(void);
rb_encoding *rb_default_internal_encoding(void);
rb_encoding *rb_default_external_encoding(void);
int rb_ascii8bit_encindex(void);
int rb_utf8_encindex(void);
int rb_usascii_encindex(void);
int rb_locale_encindex(void);
int rb_filesystem_encindex(void);

VALUE rb_enc_str_new(const char *text, long length, rb_encoding *encoding);
VALUE rb_enc_str_buf_cat(VALUE string, const char *text, long length, rb_encoding *encoding);
VALUE rb_external_str_new_with_enc(const char *text, long length, rb_encoding *encoding);
VALUE rb_enc_interned_str(const char *text, long length, rb_encoding *encoding);
VALUE rb_enc_interned_str_cstr(const char *text, rb_encoding *encoding);
VALUE rb_str_conv_enc(VALUE string, rb_encoding *from, rb_encoding *to);
VALUE rb_str_conv_enc_opts(VALUE string, rb_encoding *from, rb_encoding *to, int flags, VALUE options);
VALUE rb_str_export_to_enc(VALUE string, rb_encoding *encoding);
VALUE rb_enc_str_new_cstr(const char *text, rb_encoding *encoding);
VALUE rb_enc_str_new_static(const char *text, long length, rb_encoding *encoding);
int rb_enc_codelen(int code, rb_encoding *encoding);
int rb_enc_mbcput(unsigned int code, void *buffer, rb_encoding *encoding);
long rb_enc_strlen(const char *start, const char *end, rb_encoding *encoding);
unsigned int rb_enc_mbc_to_codepoint(const char *start, const char *end, rb_encoding *encoding);
int rb_enc_precise_mbclen(const char *start, const char *end, rb_encoding *encoding);
char *rb_enc_nth(const char *start, const char *end, long index, rb_encoding *encoding);
unsigned int rb_enc_codepoint_len(const char *start, const char *end, int *length, rb_encoding *encoding);
char *rb_enc_left_char_head(const char *start, const char *at, const char *end, rb_encoding *encoding);
int rb_enc_isalnum(int code, rb_encoding *encoding);
int rb_enc_isspace(int code, rb_encoding *encoding);
int rb_uv_to_utf8(char buffer[6], unsigned long code);

NORETURN(void rb_metorex_enc_raise(rb_encoding *encoding, VALUE klass, VALUE message));
NORETURN(static inline void rb_enc_raise(rb_encoding *encoding, VALUE klass, const char *format, ...));
static inline void rb_enc_raise(rb_encoding *encoding, VALUE klass, const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE message = rb_vsprintf(format, arguments);
  va_end(arguments);
  rb_metorex_enc_raise(encoding, klass, message);
}

typedef unsigned char OnigUChar;
#define ONIGENC_CODE_TO_MBC_MAXLEN 7
#define ONIGENC_GET_CASE_FOLD_CODES_MAX_NUM 13
#define ONIGENC_CASE_FOLD (1 << 30)
int rb_metorex_mbc_case_fold(rb_encoding *encoding, int flag, const OnigUChar **at, const OnigUChar *end,
                             OnigUChar *folded);
#define ONIGENC_MBC_CASE_FOLD(encoding, flag, at, end, folded) \
  rb_metorex_mbc_case_fold((encoding), (flag), (const OnigUChar **)(at), (end), (folded))
int rb_metorex_is_unicode(rb_encoding *encoding);
#define ONIGENC_IS_UNICODE(encoding) rb_metorex_is_unicode(encoding)

ID rb_intern3(const char *text, long length, rb_encoding *encoding);
VALUE rb_check_symbol_cstr(const char *text, long length, rb_encoding *encoding);

#ifdef __cplusplus
}
#endif

#endif
