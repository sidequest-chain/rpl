//! Embedded C99 runtime header for compiled RPL programs.

/// The complete C99 runtime header included at the top of generated C source files.
pub const RPL_RUNTIME_H: &str = r#"/* === Running Pseudo Language (RPL) C99 Runtime Header === */
#ifndef RPL_RUNTIME_H
#define RPL_RUNTIME_H

#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>

#ifdef _WIN32
  #define RPL_INLINE static __inline
#else
  #define RPL_INLINE static inline
#endif

/* --- First-Class Ternary Logic (Trit) --- */
typedef enum {
    RPL_TRIT_FALSE   = -1,
    RPL_TRIT_UNKNOWN =  0,
    RPL_TRIT_TRUE    =  1
} rpl_trit_t;

/* Kleene Ternary AND */
RPL_INLINE rpl_trit_t rpl_trit_and(rpl_trit_t a, rpl_trit_t b) {
    if (a == RPL_TRIT_FALSE || b == RPL_TRIT_FALSE) {
        return RPL_TRIT_FALSE;
    }
    if (a == RPL_TRIT_TRUE && b == RPL_TRIT_TRUE) {
        return RPL_TRIT_TRUE;
    }
    return RPL_TRIT_UNKNOWN;
}

/* Kleene Ternary OR */
RPL_INLINE rpl_trit_t rpl_trit_or(rpl_trit_t a, rpl_trit_t b) {
    if (a == RPL_TRIT_TRUE || b == RPL_TRIT_TRUE) {
        return RPL_TRIT_TRUE;
    }
    if (a == RPL_TRIT_FALSE && b == RPL_TRIT_FALSE) {
        return RPL_TRIT_FALSE;
    }
    return RPL_TRIT_UNKNOWN;
}

/* Kleene Ternary NOT */
RPL_INLINE rpl_trit_t rpl_trit_not(rpl_trit_t a) {
    if (a == RPL_TRIT_TRUE) return RPL_TRIT_FALSE;
    if (a == RPL_TRIT_FALSE) return RPL_TRIT_TRUE;
    return RPL_TRIT_UNKNOWN;
}

/* Trit string representation */
RPL_INLINE const char* rpl_trit_to_str(rpl_trit_t t) {
    switch (t) {
        case RPL_TRIT_TRUE:    return "true";
        case RPL_TRIT_FALSE:   return "false";
        case RPL_TRIT_UNKNOWN: 
        default:               return "unknown";
    }
}

/* --- Built-in Output Functions --- */
RPL_INLINE void print_int(int64_t v) {
    printf("%" PRId64, v);
    fflush(stdout);
}

RPL_INLINE void print_float(double v) {
    printf("%g", v);
    fflush(stdout);
}

RPL_INLINE void print_bool(bool v) {
    printf("%s", v ? "true" : "false");
    fflush(stdout);
}

RPL_INLINE void print_trit(rpl_trit_t v) {
    printf("%s", rpl_trit_to_str(v));
    fflush(stdout);
}

RPL_INLINE void print_str(const char* s) {
    if (s) {
        fputs(s, stdout);
        fflush(stdout);
    }
}

RPL_INLINE void println(const char* s) {
    if (s) {
        puts(s);
    } else {
        putchar('\n');
    }
    fflush(stdout);
}

RPL_INLINE void print(const char* s) {
    println(s);
}

/* --- String Helpers for Interpolation --- */
RPL_INLINE char* rpl_str_concat(const char* a, const char* b) {
    size_t la = a ? strlen(a) : 0;
    size_t lb = b ? strlen(b) : 0;
    char* res = (char*)malloc(la + lb + 1);
    if (!res) return "";
    if (a) memcpy(res, a, la);
    if (b) memcpy(res + la, b, lb);
    res[la + lb] = '\0';
    return res;
}

RPL_INLINE char* rpl_int_to_str(int64_t v) {
    char* buf = (char*)malloc(32);
    if (!buf) return "";
    snprintf(buf, 32, "%" PRId64, v);
    return buf;
}

RPL_INLINE char* rpl_float_to_str(double v) {
    char* buf = (char*)malloc(64);
    if (!buf) return "";
    snprintf(buf, 64, "%g", v);
    return buf;
}

RPL_INLINE const char* rpl_bool_to_str(bool v) {
    return v ? "true" : "false";
}

/* --- System Resource Handles --- */
typedef FILE* rpl_file_t;

/* --- Layer 1: Zero-Ceremony Convenience I/O --- */
RPL_INLINE char* rpl_input(void) {
    char buf[4096];
    if (!fgets(buf, sizeof(buf), stdin)) {
        char* empty = (char*)malloc(1);
        if (empty) empty[0] = '\0';
        return empty ? empty : "";
    }
    size_t len = strlen(buf);
    while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r')) {
        buf[--len] = '\0';
    }
    char* res = (char*)malloc(len + 1);
    if (!res) return "";
    memcpy(res, buf, len + 1);
    return res;
}

RPL_INLINE char* rpl_read_file(const char* path) {
    if (!path) return "";
    FILE* f = fopen(path, "rb");
    if (!f) return "";
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return "";
    }
    long size = ftell(f);
    if (size < 0) {
        fclose(f);
        return "";
    }
    rewind(f);
    char* buf = (char*)malloc((size_t)size + 1);
    if (!buf) {
        fclose(f);
        return "";
    }
    size_t read_bytes = fread(buf, 1, (size_t)size, f);
    buf[read_bytes] = '\0';
    fclose(f);
    return buf;
}

RPL_INLINE bool rpl_write_file(const char* path, const char* content) {
    if (!path) return false;
    FILE* f = fopen(path, "wb");
    if (!f) return false;
    if (content) {
        fputs(content, f);
    }
    fclose(f);
    return true;
}

RPL_INLINE bool rpl_append_file(const char* path, const char* content) {
    if (!path) return false;
    FILE* f = fopen(path, "ab");
    if (!f) return false;
    if (content) {
        fputs(content, f);
    }
    fclose(f);
    return true;
}

/* --- Layer 2: System Streams / Handles --- */
RPL_INLINE rpl_file_t rpl_open_file(const char* path, const char* mode) {
    if (!path || !mode) return NULL;
    return fopen(path, mode);
}

RPL_INLINE char* rpl_read_line(rpl_file_t file) {
    if (!file) return "";
    char buf[4096];
    if (!fgets(buf, sizeof(buf), file)) {
        return "";
    }
    size_t len = strlen(buf);
    while (len > 0 && (buf[len - 1] == '\n' || buf[len - 1] == '\r')) {
        buf[--len] = '\0';
    }
    char* res = (char*)malloc(len + 1);
    if (!res) return "";
    memcpy(res, buf, len + 1);
    return res;
}

RPL_INLINE bool rpl_write_line(rpl_file_t file, const char* line) {
    if (!file) return false;
    if (line) {
        fputs(line, file);
    }
    fputc('\n', file);
    fflush(file);
    return true;
}

RPL_INLINE void rpl_close_file(rpl_file_t file) {
    if (file) {
        fclose(file);
    }
}

#endif /* RPL_RUNTIME_H */
"#;
