#ifndef RUBY_INTERNAL_FORMAT_H
#define RUBY_INTERNAL_FORMAT_H 1

/* The printf formatting behind rb_raise, rb_sprintf, rb_str_catf and
 * rb_warn. A conversion written with PRIsVALUE takes a VALUE and writes
 * what its to_s answers, or what inspect answers with the '+' flag. Every
 * other conversion is handed to snprintf. */

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PRI_VALUE_PREFIX "l"
#define RUBY_PRI_VALUE_MARK "\v"
#define PRIsVALUE PRI_VALUE_PREFIX "i" RUBY_PRI_VALUE_MARK
#define PRIdVALUE PRI_VALUE_PREFIX "d"
#define PRIiVALUE PRI_VALUE_PREFIX "i"
#define PRIxVALUE PRI_VALUE_PREFIX "x"
#define PRI_SIZE_PREFIX "z"

/* The text written so far, and the last string a VALUE conversion wrote,
 * whose encoding the result takes when the two are compatible. */
typedef struct {
  char *text;
  size_t length;
  size_t capacity;
  VALUE encoding_source;
} metorex_format_buffer;

static inline void metorex_format_append(metorex_format_buffer *buffer, const char *text, size_t length) {
  if (buffer->length + length + 1 > buffer->capacity) {
    size_t wanted = (buffer->length + length + 1) * 2;
    buffer->text = (char *)realloc(buffer->text, wanted);
    buffer->capacity = wanted;
  }
  memcpy(buffer->text + buffer->length, text, length);
  buffer->length += length;
  buffer->text[buffer->length] = '\0';
}

/* Formats one conversion, `spec` with any '*' already replaced by numbers,
 * taking its argument from `arguments`. */
#define METOREX_FORMAT_ONE(type)                                         \
  do {                                                                   \
    type held = va_arg(*arguments, type);                                \
    int needed = snprintf(NULL, 0, spec, held);                          \
    char *written = (char *)malloc((size_t)needed + 1);                  \
    snprintf(written, (size_t)needed + 1, spec, held);                   \
    metorex_format_append(buffer, written, (size_t)needed);              \
    free(written);                                                       \
  } while (0)

static inline void metorex_vformat(metorex_format_buffer *buffer, const char *format, va_list *arguments) {
  const char *cursor = format;
  while (*cursor) {
    if (*cursor != '%') {
      const char *start = cursor;
      while (*cursor && *cursor != '%') cursor++;
      metorex_format_append(buffer, start, (size_t)(cursor - start));
      continue;
    }
    cursor++;
    if (*cursor == '%') {
      metorex_format_append(buffer, "%", 1);
      cursor++;
      continue;
    }
    char spec[64];
    size_t spec_length = 0;
    int inspect = 0;
    spec[spec_length++] = '%';
    while (*cursor && strchr("-+ #0", *cursor)) {
      if (*cursor == '+') inspect = 1;
      if (spec_length < 40) spec[spec_length++] = *cursor;
      cursor++;
    }
    if (*cursor == '*') {
      spec_length += (size_t)snprintf(spec + spec_length, sizeof(spec) - spec_length, "%d", va_arg(*arguments, int));
      cursor++;
    } else {
      while (*cursor >= '0' && *cursor <= '9' && spec_length < 50) spec[spec_length++] = *cursor++;
    }
    if (*cursor == '.') {
      spec[spec_length++] = *cursor++;
      if (*cursor == '*') {
        spec_length += (size_t)snprintf(spec + spec_length, sizeof(spec) - spec_length, "%d", va_arg(*arguments, int));
        cursor++;
      } else {
        while (*cursor >= '0' && *cursor <= '9' && spec_length < 58) spec[spec_length++] = *cursor++;
      }
    }
    char size[3] = {0, 0, 0};
    size_t size_length = 0;
    while (*cursor && strchr("hlLqjzt", *cursor) && size_length < 2) {
      size[size_length++] = *cursor;
      spec[spec_length++] = *cursor++;
    }
    char conversion = *cursor;
    if (!conversion) break;
    cursor++;
    if (conversion == 'i' && *cursor == '\v') {
      cursor++;
      VALUE held = va_arg(*arguments, VALUE);
      VALUE text = inspect ? rb_inspect(held) : rb_obj_as_string(held);
      /* The width and precision written with it apply to the text, so it
       * is formatted with them as a %s conversion, without the '+' that
       * asked for inspect. */
      char text_spec[64];
      size_t text_length = 0;
      for (size_t index = 0; index < spec_length - size_length; index++) {
        if (spec[index] != '+') text_spec[text_length++] = spec[index];
      }
      text_spec[text_length++] = 's';
      text_spec[text_length] = '\0';
      if (text_length == 2) {
        metorex_format_append(buffer, RSTRING_PTR(text), (size_t)RSTRING_LEN(text));
      } else {
        int needed = snprintf(NULL, 0, text_spec, RSTRING_PTR(text));
        char *written = (char *)malloc((size_t)needed + 1);
        snprintf(written, (size_t)needed + 1, text_spec, RSTRING_PTR(text));
        metorex_format_append(buffer, written, (size_t)needed);
        free(written);
      }
      buffer->encoding_source = text;
      continue;
    }
    spec[spec_length++] = conversion;
    spec[spec_length] = '\0';
    switch (conversion) {
      case 'd':
      case 'i':
        if (!strcmp(size, "ll") || !strcmp(size, "q")) METOREX_FORMAT_ONE(long long);
        else if (!strcmp(size, "l") || !strcmp(size, "z") || !strcmp(size, "t") || !strcmp(size, "j")) METOREX_FORMAT_ONE(long);
        else METOREX_FORMAT_ONE(int);
        break;
      case 'u':
      case 'o':
      case 'x':
      case 'X':
        if (!strcmp(size, "ll") || !strcmp(size, "q")) METOREX_FORMAT_ONE(unsigned long long);
        else if (!strcmp(size, "l") || !strcmp(size, "z") || !strcmp(size, "t") || !strcmp(size, "j")) METOREX_FORMAT_ONE(unsigned long);
        else METOREX_FORMAT_ONE(unsigned int);
        break;
      case 'c':
        METOREX_FORMAT_ONE(int);
        break;
      case 's':
        METOREX_FORMAT_ONE(const char *);
        break;
      case 'p':
        METOREX_FORMAT_ONE(void *);
        break;
      case 'f':
      case 'F':
      case 'e':
      case 'E':
      case 'g':
      case 'G':
      case 'a':
      case 'A':
        if (!strcmp(size, "L")) METOREX_FORMAT_ONE(long double);
        else METOREX_FORMAT_ONE(double);
        break;
      default:
        /* An unknown conversion writes its own letter, as MRI's
         * formatter does. */
        metorex_format_append(buffer, &conversion, 1);
        break;
    }
  }
}

#undef METOREX_FORMAT_ONE

static inline VALUE rb_vsprintf(const char *format, va_list arguments) {
  metorex_format_buffer buffer = {NULL, 0, 0, Qnil};
  va_list copied;
  va_copy(copied, arguments);
  metorex_vformat(&buffer, format, &copied);
  va_end(copied);
  VALUE made = rb_str_new(buffer.text ? buffer.text : "", (long)buffer.length);
  free(buffer.text);
  if (!NIL_P(buffer.encoding_source)) rb_metorex_adopt_encoding(made, buffer.encoding_source);
  return made;
}

static inline VALUE rb_sprintf(const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE made = rb_vsprintf(format, arguments);
  va_end(arguments);
  return made;
}

static inline VALUE rb_str_vcatf(VALUE string, const char *format, va_list arguments) {
  return rb_str_append(string, rb_vsprintf(format, arguments));
}

static inline VALUE rb_str_catf(VALUE string, const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE made = rb_vsprintf(format, arguments);
  va_end(arguments);
  return rb_str_append(string, made);
}

NORETURN(static inline void rb_raise(VALUE klass, const char *format, ...));
static inline void rb_raise(VALUE klass, const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE message = rb_vsprintf(format, arguments);
  va_end(arguments);
  rb_exc_raise(rb_exc_new_str(klass, message));
}

static inline void rb_warn(const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE message = rb_vsprintf(format, arguments);
  va_end(arguments);
  rb_warn_message(message, 0);
}

static inline void rb_category_warn(rb_warning_category_t category, const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE message = rb_vsprintf(format, arguments);
  va_end(arguments);
  rb_metorex_category_warn(category, message);
}

static inline void rb_warning(const char *format, ...) {
  va_list arguments;
  va_start(arguments, format);
  VALUE message = rb_vsprintf(format, arguments);
  va_end(arguments);
  rb_warn_message(message, 1);
}

#endif
