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
    uint64_t x = 0xD6E8FEB86659FD93ULL ^ ((uint64_t)base << 56) ^ index;
    size_t i;
    for (i = 0; i < 48; i++) {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        seed[i] = (uint8_t)(x >> ((i & 7u) * 8u));
    }
}

static void
fill_msg(uint8_t msg[MSG_LEN], uint8_t tweak)
{
    size_t i;
    for (i = 0; i < MSG_LEN; i++) {
        msg[i] = (uint8_t)(0xA5u ^ tweak ^ (uint8_t)(i * 17u));
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
    uint8_t key_seed[48];
    uint8_t sign_seed[48];
    uint8_t msg_valid[MSG_LEN];
    uint8_t msg_invalid[MSG_LEN];
    uint8_t sk[4096];
    uint8_t pk[2048];
    uint8_t sig[FF_FALCON_SIG_MAX];
    size_t sk_len = sizeof sk;
    size_t pk_len = sizeof pk;
    size_t sig_len = sizeof sig;
    stats_t valid_stats;
    stats_t invalid_stats;
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

    fill_seed(key_seed, 0x19, 0);
    fill_seed(sign_seed, 0xC3, 0);
    fill_msg(msg_valid, 0x00);
    fill_msg(msg_invalid, 0x5A);

    if (ff_falcon_keygen_seeded(key_seed, sizeof key_seed,
                                sk, &sk_len,
                                pk, &pk_len) != 0) {
        fprintf(stderr, "ff_falcon_keygen_seeded failed\n");
        return 1;
    }
    if (ff_falcon_sign_ct_seeded(sign_seed, sizeof sign_seed,
                                 sk, sk_len,
                                 msg_valid, sizeof msg_valid,
                                 sig, &sig_len) != 0) {
        fprintf(stderr, "ff_falcon_sign_ct_seeded failed\n");
        return 1;
    }
    if (ff_falcon_verify(pk, pk_len, msg_valid, sizeof msg_valid, sig, sig_len) != 0) {
        fprintf(stderr, "baseline valid verify failed\n");
        return 1;
    }
    if (ff_falcon_verify(pk, pk_len, msg_invalid, sizeof msg_invalid, sig, sig_len) == 0) {
        fprintf(stderr, "baseline invalid verify unexpectedly passed\n");
        return 1;
    }

    memset(&valid_stats, 0, sizeof valid_stats);
    memset(&invalid_stats, 0, sizeof invalid_stats);

    for (i = 0; i < samples; i++) {
        const uint8_t *msg;
        uint64_t start_ns;
        uint64_t end_ns;
        int rc;

        if ((i & 1u) == 0u) {
            msg = msg_valid;
        } else {
            msg = msg_invalid;
        }

        start_ns = now_ns();
        rc = ff_falcon_verify(pk, pk_len, msg, MSG_LEN, sig, sig_len);
        end_ns = now_ns();

        if ((i & 1u) == 0u) {
            if (rc != 0) {
                fprintf(stderr, "valid verify failed at sample %u\n", i);
                return 1;
            }
            stats_push(&valid_stats, (double)(end_ns - start_ns));
        } else {
            if (rc == 0) {
                fprintf(stderr, "invalid verify passed at sample %u\n", i);
                return 1;
            }
            stats_push(&invalid_stats, (double)(end_ns - start_ns));
        }
    }

    printf("samples_valid=%llu\n", (unsigned long long)valid_stats.n);
    printf("samples_invalid=%llu\n", (unsigned long long)invalid_stats.n);
    printf("mean_valid_ns=%.2f\n", valid_stats.mean);
    printf("mean_invalid_ns=%.2f\n", invalid_stats.mean);
    printf("var_valid=%.2f\n", stats_var(&valid_stats));
    printf("var_invalid=%.2f\n", stats_var(&invalid_stats));
    printf("welch_t=%.6f\n", welch_t(&valid_stats, &invalid_stats));
    return 0;
}
