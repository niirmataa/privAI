#pragma once
#include <stddef.h>
#include <stdint.h>

// Falcon round3 reference (Extra/c) wrapper.

#define FF_FALCON_LOGN 10
#define FF_FALCON_SIG_MAX 4096
#define FF_FALCON_N (1u << FF_FALCON_LOGN)

typedef struct ff_falcon_signer_ctx ff_falcon_signer_ctx;

int ff_falcon_keygen(uint8_t *sk, size_t *sk_len, uint8_t *pk, size_t *pk_len);

#if defined(NXMS_FALCON_AUDIT_RAW_API) && NXMS_FALCON_AUDIT_RAW_API
/* Audit/test-only raw signing surface. Runtime must use signer ctx. */
int ff_falcon_sign_ct(const uint8_t *sk, size_t sk_len,
                      const uint8_t *msg, size_t msg_len,
                      uint8_t *sig, size_t *sig_len);
#endif

ff_falcon_signer_ctx *ff_falcon_signer_ctx_new(const uint8_t *sk, size_t sk_len);
void ff_falcon_signer_ctx_free(ff_falcon_signer_ctx *ctx);
int ff_falcon_sign_ct_with_ctx(const ff_falcon_signer_ctx *ctx,
                               const uint8_t *msg, size_t msg_len,
                               uint8_t *sig, size_t *sig_len);

int ff_falcon_verify(const uint8_t *pk, size_t pk_len,
                     const uint8_t *msg, size_t msg_len,
                     const uint8_t *sig, size_t sig_len);

// SHAKE256 KDF helper for producing `out_len` bytes.
void ff_shake256_kdf(const uint8_t *in, size_t in_len,
                     const uint8_t *ctx, size_t ctx_len,
                     uint8_t *out, size_t out_len);
