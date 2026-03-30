#define _GNU_SOURCE
#include "pqc_falcon.h"

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define DEFAULT_SAMPLES 4096u
#define MSG_LEN 64u

typedef struct {
    uint64_t n;
    double mean;
    double m2;
} stats_t;

static void
fill_seed(uint8_t seed[48], uint8_t base, uint64_t index)
{
    uint64_t x = 0x9E3779B97F4A7C15ULL ^ ((uint64_t)base << 56) ^ index;
    size_t i;
    for (i = 0; i < 48; i++) {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        seed[i] = (uint8_t)(x >> ((i & 7u) * 8u));
    }
}

static void
fill_msg(uint8_t msg[MSG_LEN])
{
    size_t i;
    for (i = 0; i < MSG_LEN; i++) {
        msg[i] = (uint8_t)(0x3Cu ^ (uint8_t)(i * 29u));
    }
}

static uint64_t
now_ns(void)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC_RAW, &ts) != 0) {
        return 0;
    }
    return ((uint64_t)ts.tv_sec * 1000000000ULL) + (uint64_t)ts.tv_nsec;
}

static void
stats_push(stats_t *st, double sample)
{
    double delta;
    double delta2;

    st->n += 1;
    delta = sample - st->mean;
    st->mean += delta / (double)st->n;
    delta2 = sample - st->mean;
    st->m2 += delta * delta2;
}

static double
stats_var(const stats_t *st)
{
    if (st->n < 2) {
        return 0.0;
    }
    return st->m2 / (double)(st->n - 1);
}

static double
welch_t(const stats_t *a, const stats_t *b)
{
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

int
main(int argc, char **argv)
{
    uint32_t samples = DEFAULT_SAMPLES;
    uint8_t key_seed_a[48];
    uint8_t key_seed_b[48];
    uint8_t sign_seed[48];
    uint8_t msg[MSG_LEN];
    uint8_t sk_a[4096];
    uint8_t sk_b[4096];
    uint8_t pk_a[2048];
    uint8_t pk_b[2048];
    size_t sk_a_len = sizeof sk_a;
    size_t sk_b_len = sizeof sk_b;
    size_t pk_a_len = sizeof pk_a;
    size_t pk_b_len = sizeof pk_b;
    stats_t a_stats;
    stats_t b_stats;
    uint32_t i;

    if (argc == 2) {
        char *endptr = NULL;
        unsigned long parsed = strtoul(argv[1], &endptr, 10);
        if (endptr && *endptr == '\0' && parsed > 0UL && parsed <= 10000000UL) {
            samples = (uint32_t)parsed;
        }
    } else if (argc > 2) {
        fprintf(stderr, "usage: %s [samples]\n", argv[0]);
        return 2;
    }

    fill_seed(key_seed_a, 0x31, 0);
    fill_seed(key_seed_b, 0x73, 0);
    fill_msg(msg);

    if (ff_falcon_keygen_seeded(key_seed_a, sizeof key_seed_a,
                                sk_a, &sk_a_len,
                                pk_a, &pk_a_len) != 0) {
        fprintf(stderr, "ff_falcon_keygen_seeded A failed\n");
        return 1;
    }
    if (ff_falcon_keygen_seeded(key_seed_b, sizeof key_seed_b,
                                sk_b, &sk_b_len,
                                pk_b, &pk_b_len) != 0) {
        fprintf(stderr, "ff_falcon_keygen_seeded B failed\n");
        return 1;
    }

    memset(&a_stats, 0, sizeof a_stats);
    memset(&b_stats, 0, sizeof b_stats);

    for (i = 0; i < samples; i++) {
        const uint8_t *sk;
        size_t sk_len;
        uint8_t sig[FF_FALCON_SIG_MAX];
        size_t sig_len = sizeof sig;
        uint64_t start_ns;
        uint64_t end_ns;
        int rc;

        fill_seed(sign_seed, 0x55, (uint64_t)i);
        if ((i & 1u) == 0u) {
            sk = sk_a;
            sk_len = sk_a_len;
        } else {
            sk = sk_b;
            sk_len = sk_b_len;
        }

        start_ns = now_ns();
        rc = ff_falcon_sign_ct_seeded(sign_seed, sizeof sign_seed,
                                      sk, sk_len,
                                      msg, sizeof msg,
                                      sig, &sig_len);
        end_ns = now_ns();
        if (rc != 0) {
            fprintf(stderr, "ff_falcon_sign_ct_seeded failed at sample %u\n", i);
            return 1;
        }

        if ((i & 1u) == 0u) {
            stats_push(&a_stats, (double)(end_ns - start_ns));
        } else {
            stats_push(&b_stats, (double)(end_ns - start_ns));
        }
    }

    printf("samples_key_a=%llu\n", (unsigned long long)a_stats.n);
    printf("samples_key_b=%llu\n", (unsigned long long)b_stats.n);
    printf("mean_key_a_ns=%.2f\n", a_stats.mean);
    printf("mean_key_b_ns=%.2f\n", b_stats.mean);
    printf("var_key_a=%.2f\n", stats_var(&a_stats));
    printf("var_key_b=%.2f\n", stats_var(&b_stats));
    printf("welch_t=%.6f\n", welch_t(&a_stats, &b_stats));
    return 0;
}
