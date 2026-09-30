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

#endif /* RPL_RUNTIME_H */
"#;
