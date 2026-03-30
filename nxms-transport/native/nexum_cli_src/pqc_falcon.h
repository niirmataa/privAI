#pragma once
#include <stddef.h>
#include <stdint.h>

// Falcon round3 reference (Extra/c) wrapper.

#define FF_FALCON_LOGN 10
#define FF_FALCON_SIG_MAX 4096
#define FF_FALCON_N (1u << FF_FALCON_LOGN)

typedef struct {
    uint32_t magic;
    uint32_t logn;
    int8_t f[FF_FALCON_N];
    int8_t g[FF_FALCON_N];
    int8_t F[FF_FALCON_N];
    int8_t G[FF_FALCON_N];
} ff_falcon_prepared_sk_t;

int ff_falcon_keygen(uint8_t *sk, size_t *sk_len, uint8_t *pk, size_t *pk_len);

/*
 * Audit/test support: deterministic seeded variants for reproducible
 * correctness checks and KAT-style vectors. These use the same Falcon CT
 * path as the production wrapper, but with explicit 48-byte RNG seed
 * material instead of OS entropy.
 */
int ff_falcon_keygen_seeded(const uint8_t *seed, size_t seed_len,
                            uint8_t *sk, size_t *sk_len,
                            uint8_t *pk, size_t *pk_len);

// CT signature. Caller provides sig buffer and sets *sig_len to max.
int ff_falcon_sign_ct(const uint8_t *sk, size_t sk_len,
                      const uint8_t *msg, size_t msg_len,
                      uint8_t *sig, size_t *sig_len);

int ff_falcon_sign_ct_seeded(const uint8_t *seed, size_t seed_len,
                             const uint8_t *sk, size_t sk_len,
                             const uint8_t *msg, size_t msg_len,
                             uint8_t *sig, size_t *sig_len);

/*
 * Prepared-key CT sign path for audits and long-lived signer processes.
 * This keeps the reference Falcon sign_dyn algorithm, but decodes the
 * encoded secret key once and reuses the active signing lane.
 */
int ff_falcon_prepare_sk(const uint8_t *sk, size_t sk_len,
                         ff_falcon_prepared_sk_t *prepared);
void ff_falcon_clear_prepared_sk(ff_falcon_prepared_sk_t *prepared);
int ff_falcon_sign_ct_prepared(const ff_falcon_prepared_sk_t *prepared,
                               const uint8_t *msg, size_t msg_len,
                               uint8_t *sig, size_t *sig_len);
int ff_falcon_sign_ct_prepared_seeded(const uint8_t *seed, size_t seed_len,
                                      const ff_falcon_prepared_sk_t *prepared,
                                      const uint8_t *msg, size_t msg_len,
                                      uint8_t *sig, size_t *sig_len);

int ff_falcon_verify(const uint8_t *pk, size_t pk_len,
                     const uint8_t *msg, size_t msg_len,
                     const uint8_t *sig, size_t sig_len);

// SHAKE256 KDF helper for producing `out_len` bytes.
void ff_shake256_kdf(const uint8_t *in, size_t in_len,
                     const uint8_t *ctx, size_t ctx_len,
                     uint8_t *out, size_t out_len);
