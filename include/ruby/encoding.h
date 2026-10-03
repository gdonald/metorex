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

ID rb_intern3(const char *text, long length, rb_encoding *encoding);
VALUE rb_check_symbol_cstr(const char *text, long length, rb_encoding *encoding);

#ifdef __cplusplus
}
#endif

#endif
