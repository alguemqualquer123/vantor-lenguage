//! Embedded C runtime for the native backend.
//!
//! Every function here mirrors a piece of the reference interpreter
//! (`lexicon-cli/src/interp.rs`) byte-for-byte in observable behavior:
//! step accounting (50M), output buffering + cap (1M bytes), display /
//! debug / inspect rendering, Rust `{:?}` escaping, Rust `Display` for
//! f64 (shortest roundtrip, positional — never exponent), Rust `str`
//! parsing for casts, and the exact runtime error strings.

pub const RUNTIME: &str = r##"
#ifdef _WIN32
#define _CRT_SECURE_NO_WARNINGS 1
#endif
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>
#include <math.h>
#include <limits.h>

typedef long long lx_int;

/* ---------------- arena (bump allocator, never freed) ---------------- */
static char* lx_ar_cur = 0;
static size_t lx_ar_left = 0;

static void* lx_alloc(size_t n) {
    n = (n + 15u) & ~(size_t)15u;
    if (n > lx_ar_left) {
        size_t blk = n > (1u << 20) ? n : (1u << 20);
        char* p = (char*)malloc(blk);
        if (!p) { fputs("out of memory\n", stderr); exit(2); }
        lx_ar_cur = p;
        lx_ar_left = blk;
    }
    {
        char* r = lx_ar_cur;
        lx_ar_cur += n;
        lx_ar_left -= n;
        return r;
    }
}

/* ---------------- status: steps, depth, output ---------------- */
#define LX_BUDGET 50000000LL
#define LX_MAXOUT 1000000LL
#define LX_MAXDEPTH 500LL

static long long lx_steps = 0;
static long long lx_depth = 0;

static char* lx_out_p = 0;
static long long lx_out_n = 0;
static long long lx_out_c = 0;

static void lx_panic(const char* msg) {
    fputs(msg, stderr);
    fputc('\n', stderr);
    exit(1);
}

static void lx_panicf(const char* fmt, ...) {
    char buf[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    lx_panic(buf);
}

static void lx_tick(void) {
    lx_steps += 1;
    if (lx_steps > LX_BUDGET) lx_panic("execution limit exceeded (possible infinite loop)");
}

static void lx_opush(const char* s, long long n) {
    lx_tick();
    if (lx_out_n + n > lx_out_c) {
        long long c = lx_out_c ? lx_out_c * 2 : 256;
        while (c < lx_out_n + n) c *= 2;
        lx_out_p = (char*)realloc(lx_out_p, (size_t)c);
        lx_out_c = c;
    }
    if (n > 0) memcpy(lx_out_p + lx_out_n, s, (size_t)n);
    lx_out_n += n;
    if (lx_out_n > LX_MAXOUT) lx_panic("output limit exceeded (possible runaway print loop)");
}

static void lx_oputs(const char* s) { lx_opush(s, (long long)strlen(s)); }

static void lx_ofinish(void) {
    if (lx_out_n > 0) {
        fwrite(lx_out_p, 1, (size_t)lx_out_n, stdout);
        fflush(stdout);
    }
}

/* ---------------- strings ---------------- */
typedef struct { const char* p; long long n; } LxStr;

static LxStr lx_s(const char* p, long long n) { LxStr s; s.p = p; s.n = n; return s; }

static LxStr lx_sn(const char* p, long long n) {
    char* d = (char*)lx_alloc(n > 0 ? (size_t)n : 1u);
    if (n > 0) memcpy(d, p, (size_t)n);
    return lx_s(d, n);
}

static LxStr lx_scat(LxStr a, LxStr b) {
    if (a.n == 0) return b;
    if (b.n == 0) return a;
    {
        char* d = (char*)lx_alloc((size_t)(a.n + b.n));
        memcpy(d, a.p, (size_t)a.n);
        memcpy(d + a.n, b.p, (size_t)b.n);
        return lx_s(d, a.n + b.n);
    }
}

static void lx_panic_s(LxStr s) {
    if (s.n > 0) fwrite(s.p, 1, (size_t)s.n, stderr);
    fputc('\n', stderr);
    exit(1);
}

typedef struct { char* p; long long n, c; } LxBuf;

static void lx_b_need(LxBuf* b, long long k) {
    if (b->n + k > b->c) {
        long long c = b->c ? b->c * 2 : 64;
        while (c < b->n + k) c *= 2;
        b->p = (char*)realloc(b->p, (size_t)c);
        b->c = c;
    }
}

static void lx_b_put(LxBuf* b, const char* s, long long n) {
    if (n <= 0) return;
    lx_b_need(b, n);
    memcpy(b->p + b->n, s, (size_t)n);
    b->n += n;
}

static void lx_b_c(LxBuf* b, char c) { lx_b_need(b, 1); b->p[b->n++] = c; }
static void lx_b_str(LxBuf* b, LxStr s) { lx_b_put(b, s.p, s.n); }
static void lx_b_i64(LxBuf* b, long long v) {
    char t[32];
    int k = snprintf(t, sizeof t, "%lld", v);
    lx_b_put(b, t, k);
}
static LxStr lx_b_fin(LxBuf* b) { return lx_s(b->p, b->n); }

/* ---------------- numbers ---------------- */
static LxStr lx_s_i64(long long v) {
    char b[32];
    int n = snprintf(b, sizeof b, "%lld", v);
    return lx_sn(b, n);
}

/* Rust `f64 as i64` (saturating, NaN -> 0). */
static long long lx_f2i(double v) {
    if (v != v) return 0;
    if (v >= 9223372036854775807.0) return 9223372036854775807LL;
    if (v <= -9223372036854775808.0) return (-9223372036854775807LL - 1);
    return (long long)v;
}

/* Rust `Display for f64`: shortest decimal that roundtrips, positional
   (never exponent), e.g. 1e21 prints as 1000000000000000000000. */
static LxStr lx_s_f64(double v) {
    char tmp[64];
    int prec;
    if (v != v) return lx_s("NaN", 3);
    if (v == (double)INFINITY) return lx_s("inf", 3);
    if (v == (double)-INFINITY) return lx_s("-inf", 4);
    for (prec = 1; prec <= 17; prec++) {
        snprintf(tmp, sizeof tmp, "%.*e", prec - 1, v);
        if (strtod(tmp, 0) == v) break;
    }
    if (prec > 17) snprintf(tmp, sizeof tmp, "%.17e", v);
    {
        char digs[32];
        int nd = 0, neg = 0, e10 = 0, point, i;
        const char* q = tmp;
        LxBuf b;
        if (*q == '-') { neg = 1; q++; }
        while (*q >= '0' && *q <= '9') { if (nd < 30) digs[nd++] = *q; q++; }
        if (*q == '.') {
            q++;
            while (*q >= '0' && *q <= '9') { if (nd < 30) digs[nd++] = *q; q++; }
        }
        if (*q == 'e' || *q == 'E') e10 = (int)strtol(q + 1, 0, 10);
        while (nd > 1 && digs[nd - 1] == '0') nd--;
        b.p = 0; b.n = 0; b.c = 0;
        if (neg) lx_b_c(&b, '-');
        point = 1 + e10;
        if (point <= 0) {
            lx_b_put(&b, "0.", 2);
            for (i = 0; i < -point; i++) lx_b_c(&b, '0');
            lx_b_put(&b, digs, nd);
        } else if (point >= nd) {
            lx_b_put(&b, digs, nd);
            for (i = nd; i < point; i++) lx_b_c(&b, '0');
        } else {
            lx_b_put(&b, digs, point);
            lx_b_c(&b, '.');
            lx_b_put(&b, digs + point, nd - point);
        }
        return lx_b_fin(&b);
    }
}

/* The interpreter's `display()` for floats: integral and finite values
   print as their saturating i64 expansion; everything else via Display. */
static LxStr lx_disp_f(double v) {
    if (v == v && v != (double)INFINITY && v != (double)-INFINITY && v == trunc(v)) {
        return lx_s_i64(lx_f2i(v));
    }
    return lx_s_f64(v);
}

/* ---------------- UTF-8 ---------------- */
static int lx_utf8_decode(const char* s, long long n, long long* pos, unsigned int* out) {
    long long i = *pos;
    unsigned char c;
    unsigned int cp;
    long long len;
    if (i >= n) return 0;
    c = (unsigned char)s[i];
    if (c < 0x80) { cp = c; len = 1; }
    else if ((c >> 5) == 6 && i + 1 < n) {
        cp = ((unsigned int)(c & 0x1F) << 6) | ((unsigned char)s[i + 1] & 0x3Fu);
        len = 2;
    } else if ((c >> 4) == 14 && i + 2 < n) {
        cp = ((unsigned int)(c & 0x0F) << 12) | (((unsigned char)s[i + 1] & 0x3Fu) << 6)
             | ((unsigned char)s[i + 2] & 0x3Fu);
        len = 3;
    } else if ((c >> 3) == 30 && i + 3 < n) {
        cp = ((unsigned int)(c & 0x07) << 18) | (((unsigned char)s[i + 1] & 0x3Fu) << 12)
             | (((unsigned char)s[i + 2] & 0x3Fu) << 6) | ((unsigned char)s[i + 3] & 0x3Fu);
        len = 4;
    } else { cp = c; len = 1; }
    *pos = i + len;
    *out = cp;
    return 1;
}

static void lx_uc_put(char* p, unsigned int c) {
    if (c < 0x80) { p[0] = (char)c; }
    else if (c < 0x800) {
        p[0] = (char)(0xC0 | (c >> 6));
        p[1] = (char)(0x80 | (c & 0x3F));
    } else if (c < 0x10000) {
        p[0] = (char)(0xE0 | (c >> 12));
        p[1] = (char)(0x80 | ((c >> 6) & 0x3F));
        p[2] = (char)(0x80 | (c & 0x3F));
    } else {
        p[0] = (char)(0xF0 | (c >> 18));
        p[1] = (char)(0x80 | ((c >> 12) & 0x3F));
        p[2] = (char)(0x80 | ((c >> 6) & 0x3F));
        p[3] = (char)(0x80 | (c & 0x3F));
    }
}

static int lx_uc_len(unsigned int c) {
    return c < 0x80 ? 1 : c < 0x800 ? 2 : c < 0x10000 ? 3 : 4;
}

static LxStr lx_s_char(unsigned int c) {
    char b[4];
    lx_uc_put(b, c);
    return lx_sn(b, lx_uc_len(c));
}

static long long lx_uchars(LxStr s) {
    long long i = 0, k = 0;
    unsigned int cp;
    while (lx_utf8_decode(s.p, s.n, &i, &cp)) k++;
    return k;
}

/* k-th char of s (already bounds-checked by callers). */
static LxStr lx_char_at_str(LxStr s, long long k) {
    long long i = 0;
    unsigned int cp = 0;
    while (k >= 0 && lx_utf8_decode(s.p, s.n, &i, &cp)) k--;
    return lx_s_char(cp);
}

static unsigned int lx_char_at_cp(LxStr s, long long k) {
    long long i = 0;
    unsigned int cp = 0;
    while (k >= 0 && lx_utf8_decode(s.p, s.n, &i, &cp)) k--;
    return cp;
}

static unsigned int lx_cast_char(LxStr s) {
    long long i = 0;
    unsigned int cp = 0;
    if (!lx_utf8_decode(s.p, s.n, &i, &cp)) return 0;
    return cp;
}

/* ---------------- Rust `{:?}` escaping ---------------- */
static LxStr lx_dbg_str(LxStr s) {
    LxBuf b;
    long long i;
    b.p = 0; b.n = 0; b.c = 0;
    lx_b_c(&b, '"');
    for (i = 0; i < s.n; i++) {
        unsigned char ch = (unsigned char)s.p[i];
        if (ch == '"') lx_b_put(&b, "\\\"", 2);
        else if (ch == '\\') lx_b_put(&b, "\\\\", 2);
        else if (ch == '\n') lx_b_put(&b, "\\n", 2);
        else if (ch == '\r') lx_b_put(&b, "\\r", 2);
        else if (ch == '\t') lx_b_put(&b, "\\t", 2);
        else if (ch == 0) lx_b_put(&b, "\\0", 2);
        else if (ch < 0x20 || ch == 0x7F) {
            char t[16];
            int k = snprintf(t, sizeof t, "\\u{%x}", (unsigned int)ch);
            lx_b_put(&b, t, k);
        } else lx_b_c(&b, (char)ch);
    }
    lx_b_c(&b, '"');
    return lx_b_fin(&b);
}

static LxStr lx_dbg_char(unsigned int c) {
    LxBuf b;
    b.p = 0; b.n = 0; b.c = 0;
    lx_b_c(&b, '\'');
    if (c == '\'') lx_b_put(&b, "\\'", 2);
    else if (c == '\\') lx_b_put(&b, "\\\\", 2);
    else if (c == '"') lx_b_put(&b, "\\\"", 2);
    else if (c == '\n') lx_b_put(&b, "\\n", 2);
    else if (c == '\r') lx_b_put(&b, "\\r", 2);
    else if (c == '\t') lx_b_put(&b, "\\t", 2);
    else if (c == 0) lx_b_put(&b, "\\0", 2);
    else if (c < 0x20 || c == 0x7F) {
        char t[16];
        int k = snprintf(t, sizeof t, "\\u{%x}", c);
        lx_b_put(&b, t, k);
    } else {
        char u[4];
        lx_uc_put(u, c);
        lx_b_put(&b, u, lx_uc_len(c));
    }
    lx_b_c(&b, '\'');
    return lx_b_fin(&b);
}

/* ---------------- Rust-style parsing (strict) ---------------- */
static int lx_pi64(LxStr s, long long* out) {
    long long i = 0;
    int neg = 0;
    unsigned long long acc, limit;
    if (s.n == 0) return 0;
    if (s.p[0] == '+' || s.p[0] == '-') { neg = (s.p[0] == '-'); i = 1; }
    if (i >= s.n) return 0;
    acc = 0;
    limit = neg ? 9223372036854775808ULL : 9223372036854775807ULL;
    for (; i < s.n; i++) {
        unsigned int d;
        if (s.p[i] < '0' || s.p[i] > '9') return 0;
        d = (unsigned int)(s.p[i] - '0');
        if (acc > (limit - d) / 10ULL) return 0;
        acc = acc * 10ULL + d;
    }
    *out = neg ? (long long)(0ULL - acc) : (long long)acc;
    return 1;
}

static int lx_ci_eq(const char* p, long long n, long long i, const char* w) {
    long long j = 0;
    for (; w[j]; j++) {
        char a;
        if (i + j >= n) return 0;
        a = p[i + j];
        if (a >= 'A' && a <= 'Z') a += 32;
        if (a != w[j]) return 0;
    }
    return (int)j;
}

static int lx_pf64(LxStr s, double* out) {
    const char* p = s.p;
    long long n = s.n, i = 0, start, fs;
    int neg = 0, any = 0;
    if (n == 0) return 0;
    if (p[i] == '+' || p[i] == '-') { neg = (p[i] == '-'); i++; }
    {
        long long k = lx_ci_eq(p, n, i, "infinity");
        if (!k) k = lx_ci_eq(p, n, i, "inf");
        if (k) {
            if (i + k != n) return 0;
            *out = neg ? (double)-INFINITY : (double)INFINITY;
            return 1;
        }
        k = lx_ci_eq(p, n, i, "nan");
        if (k) {
            if (i + k != n) return 0;
            *out = neg ? (double)-NAN : (double)NAN;
            return 1;
        }
    }
    start = i;
    while (i < n && p[i] >= '0' && p[i] <= '9') i++;
    if (i > start) any = 1;
    if (i < n && p[i] == '.') {
        i++;
        fs = i;
        while (i < n && p[i] >= '0' && p[i] <= '9') i++;
        if (i > fs) any = 1;
    }
    if (!any) return 0;
    if (i < n && (p[i] == 'e' || p[i] == 'E')) {
        long long j = i + 1, es;
        if (j < n && (p[j] == '+' || p[j] == '-')) j++;
        es = j;
        while (j < n && p[j] >= '0' && p[j] <= '9') j++;
        if (j == es) return 0;
        i = j;
    }
    if (i != n) return 0;
    {
        char* c = (char*)lx_alloc((size_t)n + 1);
        double d;
        memcpy(c, p, (size_t)n);
        c[n] = 0;
        d = strtod(c, 0);
        *out = d;
    }
    return 1;
}

/* ---------------- str helpers ---------------- */
static int lx_str_cmp(LxStr a, LxStr b) {
    long long m = a.n < b.n ? a.n : b.n;
    int c = m > 0 ? memcmp(a.p, b.p, (size_t)m) : 0;
    if (c) return c < 0 ? -1 : 1;
    return a.n < b.n ? -1 : a.n > b.n ? 1 : 0;
}

static int lx_str_eq(LxStr a, LxStr b) {
    return a.n == b.n && (a.n == 0 || memcmp(a.p, b.p, (size_t)a.n) == 0);
}

static int lx_str_contains(LxStr h, LxStr nd) {
    long long i;
    if (nd.n == 0) return 1;
    if (nd.n > h.n) return 0;
    for (i = 0; i + nd.n <= h.n; i++) {
        if (memcmp(h.p + i, nd.p, (size_t)nd.n) == 0) return 1;
    }
    return 0;
}

/* ---------------- range ---------------- */
typedef struct { long long a, b; unsigned char inc; } LxRange;

/* ---------------- lists ---------------- */
#define LX_DEF_LIST(T, M) \
typedef struct { T* d; long long n, c; } LxL_##M; \
static LxL_##M lx_l##M##_new(void) { LxL_##M l; l.d = 0; l.n = 0; l.c = 0; return l; } \
static void lx_l##M##_push(LxL_##M* l, T v) { \
    if (l->n >= l->c) { \
        long long nc = l->c ? l->c * 2 : 8; \
        T* nd = (T*)lx_alloc((size_t)nc * sizeof(T)); \
        if (l->n > 0) memcpy(nd, l->d, (size_t)l->n * sizeof(T)); \
        l->d = nd; \
        l->c = nc; \
    } \
    l->d[l->n++] = v; \
}

LX_DEF_LIST(long long, i)
LX_DEF_LIST(LxStr, str)

/* Interpreter expand_iterable() over a string: one single-char string per
   codepoint (same iteration order as Rust's `chars()`). */
static LxL_str lx_s_chars(LxStr s) {
    LxL_str out = lx_lstr_new();
    long long i = 0;
    unsigned int cp = 0;
    while (lx_utf8_decode(s.p, s.n, &i, &cp)) {
        lx_lstr_push(&out, lx_s_char(cp));
    }
    return out;
}

/* Interpreter expand_iterable() for ranges: push, guard i64::MAX, bump,
   then the 1M element cap — exact order matters for the error path. */
static LxL_i lx_range_to_list(long long a, long long b, int inc) {
    LxL_i out = lx_li_new();
    long long i = a;
    if (inc) {
        while (i <= b) {
            lx_li_push(&out, i);
            if (i == 9223372036854775807LL) break;
            i += 1;
            if (out.n > 1000000) lx_panic("range too large to iterate");
        }
    } else {
        while (i < b) {
            lx_li_push(&out, i);
            if (i == 9223372036854775807LL) break;
            i += 1;
            if (out.n > 1000000) lx_panic("range too large to iterate");
        }
    }
    return out;
}
"##;

/// Prototypes for user-facing helper families are generated after this
/// runtime; nothing here references them.
pub const RUNTIME_TAIL: &str = r##"
"##;
