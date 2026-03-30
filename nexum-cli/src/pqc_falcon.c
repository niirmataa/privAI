#include "pqc_falcon.h"
#include "util.h"

#include <sodium.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#if defined(__unix__) || defined(__APPLE__)
#include <sys/mman.h>
#endif

// Shared Falcon reference implementation from nxms-transport.
#include "../../nxms-transport/native/vendor/falcon/falcon.h"
#include "../../nxms-transport/native/vendor/falcon/inner.h"

typedef struct {
    uint32_t magic;
    uint32_t logn;
    int8_t f[FF_FALCON_N];
    int8_t g[FF_FALCON_N];
    int8_t F[FF_FALCON_N];
    int8_t G[FF_FALCON_N];
} ff_falcon_prepared_sk_t;

struct ff_falcon_signer_ctx {
    uint32_t magic;
    int is_mlocked;
    ff_falcon_prepared_sk_t prepared;
};

#define FF_FALCON_SIGNER_MAGIC 0x46464354u

static uint8_t *
ff_align_u64(void *ptr)
{
    uintptr_t x = (uintptr_t)ptr;
    x = (x + 7u) & ~(uintptr_t)7u;
    return (uint8_t *)x;
}

static int
init_rng(shake256_context *rng)
{
    int r = shake256_init_prng_from_system(rng);
    if (r < 0) {
        return r;
    }
    shake256_flip(rng);
    return 0;
}

static int
ff_falcon_prepare_sk(const uint8_t *sk, size_t sk_len,
                     ff_falcon_prepared_sk_t *prepared)
{
    size_t u, v;
    uint8_t *tmp = NULL;
    uint8_t *atmp;
    const unsigned logn = FF_FALCON_LOGN;

    if (prepared == NULL || sk == NULL) {
        return -1;
    }
    sodium_memzero(prepared, sizeof(*prepared));

    if (sk_len != FALCON_PRIVKEY_SIZE(logn) || sk_len == 0) {
        return -1;
    }
    if ((sk[0] & 0xF0) != 0x50 || (sk[0] & 0x0F) != logn) {
        return -1;
    }

    tmp = (uint8_t *)malloc(FALCON_TMPSIZE_EXPANDPRIV(logn));
    if (tmp == NULL) {
        return -1;
    }
    atmp = ff_align_u64(tmp);

    u = 1;
    v = Zf(trim_i8_decode)(prepared->f, logn, Zf(max_fg_bits)[logn],
                           sk + u, sk_len - u);
    if (v == 0) {
        goto fail;
    }
    u += v;
    v = Zf(trim_i8_decode)(prepared->g, logn, Zf(max_fg_bits)[logn],
                           sk + u, sk_len - u);
    if (v == 0) {
        goto fail;
    }
    u += v;
    v = Zf(trim_i8_decode)(prepared->F, logn, Zf(max_FG_bits)[logn],
                           sk + u, sk_len - u);
    if (v == 0) {
        goto fail;
    }
    u += v;
    if (u != sk_len) {
        goto fail;
    }
    if (!Zf(complete_private)(prepared->G,
                              prepared->f, prepared->g, prepared->F,
                              logn, atmp)) {
        goto fail;
    }

    prepared->magic = FF_FALCON_SIGNER_MAGIC;
    prepared->logn = logn;
    sodium_memzero(tmp, FALCON_TMPSIZE_EXPANDPRIV(logn));
    free(tmp);
    return 0;

fail:
    sodium_memzero(tmp, FALCON_TMPSIZE_EXPANDPRIV(logn));
    free(tmp);
    sodium_memzero(prepared, sizeof(*prepared));
    return -1;
}

static int
ff_falcon_sign_ct_prepared_with_rng(shake256_context *rng,
                                    const ff_falcon_prepared_sk_t *prepared,
                                    const uint8_t *msg, size_t msg_len,
                                    uint8_t *sig, size_t *sig_len)
{
    const unsigned logn = FF_FALCON_LOGN;
    size_t tmpsz;
    void *tmp;
    int16_t *sv;
    uint16_t *hm;
    uint8_t *atmp;
    uint8_t nonce[40];
    shake256_context hash_data;
    inner_shake256_context sav_hash_data;
    uint8_t *es;
    size_t es_len;
    size_t v;
    unsigned oldcw;
    int ret = -1;

    if (prepared == NULL || prepared->magic != FF_FALCON_SIGNER_MAGIC ||
        prepared->logn != logn) {
        return -1;
    }
    if (sig == NULL || sig_len == NULL || *sig_len < FALCON_SIG_CT_SIZE(logn)) {
        return -1;
    }

    tmpsz = FALCON_TMPSIZE_SIGNDYN(logn);
    tmp = malloc(tmpsz);
    if (tmp == NULL) {
        return -1;
    }

    if (falcon_sign_start(rng, nonce, &hash_data) != 0) {
        goto cleanup;
    }
    if (msg_len > 0) {
        shake256_inject(&hash_data, msg, msg_len);
    }

    hm = (uint16_t *)tmp;
    sv = (int16_t *)hm;
    atmp = ff_align_u64(hm + ((size_t)1 << logn));

    shake256_flip(&hash_data);
    memcpy(&sav_hash_data, &hash_data, sizeof(sav_hash_data));
    Zf(hash_to_point_ct)(&sav_hash_data, hm, logn, atmp);

    oldcw = set_fpu_cw(2);
    Zf(sign_dyn)(sv, (inner_shake256_context *)rng,
                 prepared->f, prepared->g, prepared->F, prepared->G,
                 hm, logn, atmp);
    set_fpu_cw(oldcw);

    es = sig;
    es_len = *sig_len;
    es[0] = 0x50 + logn;
    memcpy(es + 1, nonce, 40);
    v = Zf(trim_i16_encode)(es + 41, es_len - 41,
                            sv, logn, Zf(max_sig_bits)[logn]);
    if (v == 0) {
        goto cleanup;
    }

    *sig_len = 41 + v;
    ret = 0;

cleanup:
    sodium_memzero(tmp, tmpsz);
    free(tmp);
    return ret;
}

int
ff_falcon_keygen(uint8_t *sk, size_t *sk_len, uint8_t *pk, size_t *pk_len)
{
    shake256_context rng;
    size_t skmax = FALCON_PRIVKEY_SIZE(FF_FALCON_LOGN);
    size_t pkmax = FALCON_PUBKEY_SIZE(FF_FALCON_LOGN);
    void *tmp;
    int r;

    if (init_rng(&rng) != 0) {
        return -1;
    }

    tmp = malloc(FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    if (!tmp) {
        return -1;
    }

    r = falcon_keygen_make(&rng, FF_FALCON_LOGN,
                           sk, skmax,
                           pk, pkmax,
                           tmp, FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    sodium_memzero(tmp, FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    free(tmp);
    if (r != 0) {
        return -1;
    }

    *sk_len = skmax;
    *pk_len = pkmax;
    return 0;
}

#if defined(NXMS_FALCON_AUDIT_RAW_API) && NXMS_FALCON_AUDIT_RAW_API
int
ff_falcon_sign_ct(const uint8_t *sk, size_t sk_len,
                  const uint8_t *msg, size_t msg_len,
                  uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;
    int logn = falcon_get_logn((void *)sk, sk_len);
    size_t tmpsz = FALCON_TMPSIZE_SIGNDYN(FF_FALCON_LOGN);
    void *tmp;
    int r;

    if (init_rng(&rng) != 0) {
        return -1;
    }
    if (logn != FF_FALCON_LOGN) {
        return -1;
    }

    tmp = malloc(tmpsz);
    if (!tmp) {
        return -1;
    }

    r = falcon_sign_dyn(&rng,
                        sig, sig_len, FALCON_SIG_CT,
                        sk, sk_len,
                        msg, msg_len,
                        tmp, tmpsz);
    sodium_memzero(tmp, tmpsz);
    free(tmp);
    return (r == 0) ? 0 : -1;
}
#endif

ff_falcon_signer_ctx *
ff_falcon_signer_ctx_new(const uint8_t *sk, size_t sk_len)
{
    ff_falcon_signer_ctx *ctx;

    if (sk == NULL || sk_len == 0) {
        return NULL;
    }

    ctx = (ff_falcon_signer_ctx *)calloc(1, sizeof(*ctx));
    if (ctx == NULL) {
        return NULL;
    }

#if defined(__unix__) || defined(__APPLE__)
    if (mlock(ctx, sizeof(*ctx)) == 0) {
        ctx->is_mlocked = 1;
    }
#endif

    if (ff_falcon_prepare_sk(sk, sk_len, &ctx->prepared) != 0) {
        ff_falcon_signer_ctx_free(ctx);
        return NULL;
    }
    ctx->magic = FF_FALCON_SIGNER_MAGIC;
    return ctx;
}

void
ff_falcon_signer_ctx_free(ff_falcon_signer_ctx *ctx)
{
    if (ctx == NULL) {
        return;
    }
    sodium_memzero(&ctx->prepared, sizeof(ctx->prepared));
    ctx->magic = 0;
#if defined(__unix__) || defined(__APPLE__)
    if (ctx->is_mlocked) {
        sodium_memzero(ctx, sizeof(*ctx));
        munlock(ctx, sizeof(*ctx));
        free(ctx);
        return;
    }
#endif
    sodium_memzero(ctx, sizeof(*ctx));
    free(ctx);
}

int
ff_falcon_sign_ct_with_ctx(const ff_falcon_signer_ctx *ctx,
                           const uint8_t *msg, size_t msg_len,
                           uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;

    if (ctx == NULL || ctx->magic != FF_FALCON_SIGNER_MAGIC) {
        return -1;
    }
    if (init_rng(&rng) != 0) {
        return -1;
    }
    return ff_falcon_sign_ct_prepared_with_rng(&rng, &ctx->prepared, msg, msg_len, sig, sig_len);
}

int
ff_falcon_verify(const uint8_t *pk, size_t pk_len,
                 const uint8_t *msg, size_t msg_len,
                 const uint8_t *sig, size_t sig_len)
{
    int logn = falcon_get_logn((void *)pk, pk_len);
    size_t tmpsz = FALCON_TMPSIZE_VERIFY(FF_FALCON_LOGN);
    void *tmp;
    int r;

    if (logn != FF_FALCON_LOGN) {
        return -1;
    }

    tmp = malloc(tmpsz);
    if (!tmp) {
        return -1;
    }

    r = falcon_verify(sig, sig_len, FALCON_SIG_CT,
                      pk, pk_len,
                      msg, msg_len,
                      tmp, tmpsz);
    sodium_memzero(tmp, tmpsz);
    free(tmp);
    return (r == 0) ? 0 : -1;
}

void
ff_shake256_kdf(const uint8_t *in, size_t in_len,
                const uint8_t *ctx, size_t ctx_len,
                uint8_t *out, size_t out_len)
{
    shake256_context sc;
    shake256_init(&sc);
    if (ctx && ctx_len) {
        shake256_inject(&sc, ctx, ctx_len);
    }
    shake256_inject(&sc, in, in_len);
    shake256_flip(&sc);
    shake256_extract(&sc, out, out_len);
    sodium_memzero(&sc, sizeof(sc));
}
