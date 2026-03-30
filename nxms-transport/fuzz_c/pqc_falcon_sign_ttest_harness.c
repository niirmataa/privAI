#define _GNU_SOURCE
#include "pqc_falcon.h"

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define FIXTURE_MAGIC "NXMSFAL1"
#define DEFAULT_SAMPLES 4096u

typedef struct {
    uint8_t *sender_sk_sig;
    size_t sender_sk_sig_len;
    uint8_t *sender_pk_sig;
    size_t sender_pk_sig_len;
    uint8_t **messages;
    size_t *message_lens;
    size_t message_count;
} fixture_t;

typedef struct {
    uint64_t n;
    double mean;
    double m2;
} stats_t;

static void secure_free(void *ptr, size_t len) {
    volatile uint8_t *p = (volatile uint8_t *)ptr;
    if (!ptr) {
        return;
    }
    while (len--) {
        *p++ = 0;
    }
    free(ptr);
}

static uint32_t read_u32_le(const uint8_t *p) {
    return (uint32_t)p[0]
        | ((uint32_t)p[1] << 8)
        | ((uint32_t)p[2] << 16)
        | ((uint32_t)p[3] << 24);
}

static uint8_t *read_file(const char *path, size_t *out_len) {
    FILE *fp = NULL;
    uint8_t *buf = NULL;
    long file_len;

    if (!path || !out_len) {
        return NULL;
    }
    *out_len = 0;

    fp = fopen(path, "rb");
    if (!fp) {
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fclose(fp);
        return NULL;
    }
    file_len = ftell(fp);
    if (file_len < 0) {
        fclose(fp);
        return NULL;
    }
    if (fseek(fp, 0, SEEK_SET) != 0) {
        fclose(fp);
        return NULL;
    }
    buf = (uint8_t *)malloc((size_t)file_len ? (size_t)file_len : 1u);
    if (!buf) {
        fclose(fp);
        return NULL;
    }
    if ((size_t)file_len > 0 && fread(buf, 1, (size_t)file_len, fp) != (size_t)file_len) {
        free(buf);
        fclose(fp);
        return NULL;
    }
    fclose(fp);
    *out_len = (size_t)file_len;
    return buf;
}

static void fixture_free(fixture_t *fixture) {
    size_t i;

    if (!fixture) {
        return;
    }
    secure_free(fixture->sender_sk_sig, fixture->sender_sk_sig_len);
    fixture->sender_sk_sig = NULL;
    fixture->sender_sk_sig_len = 0;
    secure_free(fixture->sender_pk_sig, fixture->sender_pk_sig_len);
    fixture->sender_pk_sig = NULL;
    fixture->sender_pk_sig_len = 0;
    if (fixture->messages) {
        for (i = 0; i < fixture->message_count; i++) {
            secure_free(fixture->messages[i], fixture->message_lens ? fixture->message_lens[i] : 0);
        }
    }
    free(fixture->messages);
    fixture->messages = NULL;
    free(fixture->message_lens);
    fixture->message_lens = NULL;
    fixture->message_count = 0;
}

static int load_fixture(const char *path, fixture_t *out) {
    uint8_t *buf = NULL;
    size_t len = 0;
    size_t off = 0;
    uint32_t sk_len, pk_len, msg_count, i;

    if (!path || !out) {
        return -1;
    }
    memset(out, 0, sizeof(*out));
    buf = read_file(path, &len);
    if (!buf) {
        return -1;
    }
    if (len < 8 + 4 + 4 + 4) {
        free(buf);
        return -1;
    }
    if (memcmp(buf, FIXTURE_MAGIC, 8) != 0) {
        free(buf);
        return -1;
    }

    off = 8;
    sk_len = read_u32_le(buf + off);
    off += 4;
    if (off + sk_len + 4 > len) {
        free(buf);
        return -1;
    }
    out->sender_sk_sig = (uint8_t *)malloc(sk_len ? sk_len : 1u);
    if (!out->sender_sk_sig) {
        free(buf);
        return -1;
    }
    memcpy(out->sender_sk_sig, buf + off, sk_len);
    out->sender_sk_sig_len = sk_len;
    off += sk_len;

    pk_len = read_u32_le(buf + off);
    off += 4;
    if (off + pk_len + 4 > len) {
        fixture_free(out);
        free(buf);
        return -1;
    }
    out->sender_pk_sig = (uint8_t *)malloc(pk_len ? pk_len : 1u);
    if (!out->sender_pk_sig) {
        fixture_free(out);
        free(buf);
        return -1;
    }
    memcpy(out->sender_pk_sig, buf + off, pk_len);
    out->sender_pk_sig_len = pk_len;
    off += pk_len;

    msg_count = read_u32_le(buf + off);
    off += 4;
    out->messages = (uint8_t **)calloc(msg_count ? msg_count : 1u, sizeof(*out->messages));
    out->message_lens = (size_t *)calloc(msg_count ? msg_count : 1u, sizeof(*out->message_lens));
    if (!out->messages || !out->message_lens) {
        fixture_free(out);
        free(buf);
        return -1;
    }
    out->message_count = msg_count;

    for (i = 0; i < msg_count; i++) {
        uint32_t msg_len;
        if (off + 4 > len) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        msg_len = read_u32_le(buf + off);
        off += 4;
        if (off + msg_len > len) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        out->messages[i] = (uint8_t *)malloc(msg_len ? msg_len : 1u);
        if (!out->messages[i]) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        memcpy(out->messages[i], buf + off, msg_len);
        out->message_lens[i] = msg_len;
        off += msg_len;
    }

    free(buf);
    return 0;
}

static uint64_t xorshift64(uint64_t *state) {
    uint64_t x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    return x;
}

static void fill_seed(uint8_t seed[48], uint64_t index) {
    uint64_t state = 0x9E3779B97F4A7C15ULL ^ (index * 0xBF58476D1CE4E5B9ULL);
    size_t i;
    for (i = 0; i < 48; i++) {
        if ((i & 7u) == 0u) {
            state = xorshift64(&state);
        }
        seed[i] = (uint8_t)(state >> ((i & 7u) * 8u));
    }
}

static void fill_random_message(uint8_t *msg, size_t msg_len, uint64_t index) {
    uint64_t state = 0xD6E8FEB86659FD93ULL ^ (index * 0x94D049BB133111EBULL);
    size_t i;
    for (i = 0; i < msg_len; i++) {
        if ((i & 7u) == 0u) {
            state = xorshift64(&state);
        }
        msg[i] = (uint8_t)(state >> ((i & 7u) * 8u));
    }
}

static uint64_t now_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC_RAW, &ts) != 0) {
        return 0;
    }
    return ((uint64_t)ts.tv_sec * 1000000000ULL) + (uint64_t)ts.tv_nsec;
}

static void stats_push(stats_t *st, double sample) {
    double delta;
    double delta2;

    st->n += 1;
    delta = sample - st->mean;
    st->mean += delta / (double)st->n;
    delta2 = sample - st->mean;
    st->m2 += delta * delta2;
}

static double stats_var(const stats_t *st) {
    if (st->n < 2) {
        return 0.0;
    }
    return st->m2 / (double)(st->n - 1);
}

static double welch_t(const stats_t *a, const stats_t *b) {
    double va;
    double vb;
    double denom;

    if (a->n < 2 || b->n < 2) {
        return 0.0;
    }
    va = stats_var(a) / (double)a->n;
    vb = stats_var(b) / (double)b->n;
    denom = sqrt(va + vb);
    if (denom == 0.0) {
        return 0.0;
    }
    return (a->mean - b->mean) / denom;
}

static int run_measurement(const fixture_t *fixture, uint32_t samples) {
    const uint8_t *fixed_msg;
    size_t fixed_len;
    uint8_t *random_msg = NULL;
    uint8_t seed[48];
    uint8_t sig[FF_FALCON_SIG_MAX];
    stats_t fixed_stats;
    stats_t random_stats;
    uint32_t i;

    if (!fixture || fixture->message_count == 0) {
        return 1;
    }

    fixed_msg = fixture->messages[0];
    fixed_len = fixture->message_lens[0];
    random_msg = (uint8_t *)malloc(fixed_len ? fixed_len : 1u);
    if (!random_msg) {
        return 1;
    }

    memset(&fixed_stats, 0, sizeof fixed_stats);
    memset(&random_stats, 0, sizeof random_stats);

    for (i = 0; i < samples; i++) {
        const uint8_t *msg;
        size_t msg_len;
        uint64_t start_ns;
        uint64_t end_ns;
        size_t sig_len = sizeof(sig);
        int rc;

        fill_seed(seed, (uint64_t)i);
        if ((i & 1u) == 0u) {
            msg = fixed_msg;
            msg_len = fixed_len;
        } else {
            fill_random_message(random_msg, fixed_len, (uint64_t)i);
            msg = random_msg;
            msg_len = fixed_len;
        }

        start_ns = now_ns();
        rc = ff_falcon_sign_ct_seeded(
            seed,
            sizeof seed,
            fixture->sender_sk_sig,
            fixture->sender_sk_sig_len,
            msg,
            msg_len,
            sig,
            &sig_len
        );
        end_ns = now_ns();
        if (rc != 0) {
            secure_free(random_msg, fixed_len);
            return 1;
        }

        if ((i & 1u) == 0u) {
            stats_push(&fixed_stats, (double)(end_ns - start_ns));
        } else {
            stats_push(&random_stats, (double)(end_ns - start_ns));
        }
    }

    printf("samples_fixed=%llu\n", (unsigned long long)fixed_stats.n);
    printf("samples_random=%llu\n", (unsigned long long)random_stats.n);
    printf("mean_fixed_ns=%.2f\n", fixed_stats.mean);
    printf("mean_random_ns=%.2f\n", random_stats.mean);
    printf("var_fixed=%.2f\n", stats_var(&fixed_stats));
    printf("var_random=%.2f\n", stats_var(&random_stats));
    printf("welch_t=%.6f\n", welch_t(&fixed_stats, &random_stats));

    secure_free(random_msg, fixed_len);
    return 0;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint32_t samples = DEFAULT_SAMPLES;
    int rc;

    memset(&fixture, 0, sizeof(fixture));

    if (argc >= 3) {
        char *endptr = NULL;
        unsigned long parsed = strtoul(argv[2], &endptr, 10);
        if (endptr && *endptr == '\0' && parsed > 0UL && parsed <= 10000000UL) {
            samples = (uint32_t)parsed;
        }
    }

    if (argc < 2 || argc > 3) {
        fprintf(stderr, "usage: %s <falcon_fixture.bin> [samples]\n", argv[0]);
        return 2;
    }

    if (load_fixture(argv[1], &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", argv[1]);
        return 2;
    }

    rc = run_measurement(&fixture, samples);
    fixture_free(&fixture);
    return rc;
}
