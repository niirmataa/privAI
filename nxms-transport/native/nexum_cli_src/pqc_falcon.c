#include "pqc_falcon.h"
#include "util.h"

#include <stdlib.h>
#include <string.h>

// Vendor headers
#include "../vendor/falcon/falcon.h"
#include "../vendor/falcon/inner.h"

#if defined NXMS_CTGRIND_AUDIT && NXMS_CTGRIND_AUDIT
#include <valgrind/memcheck.h>
#endif

#define FF_FALCON_PREPARED_MAGIC 0x4650534Bu

static void
ff_secure_bzero(void *ptr, size_t len)
{
    volatile uint8_t *p = (volatile uint8_t *)ptr;
    while (len-- > 0) {
        *p++ = 0;
    }
}

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

static void
init_rng_from_seed(shake256_context *rng, const uint8_t *seed, size_t seed_len)
{
    shake256_init_prng_from_seed(rng, seed, seed_len);
    shake256_flip(rng);
}

static int
ff_falcon_prepared_is_valid(const ff_falcon_prepared_sk_t *prepared)
{
    return prepared != NULL
        && prepared->magic == FF_FALCON_PREPARED_MAGIC
        && prepared->logn == FF_FALCON_LOGN;
}

static int
ff_falcon_keygen_with_rng(shake256_context *rng,
                          uint8_t *sk, size_t *sk_len,
                          uint8_t *pk, size_t *pk_len)
{
    size_t skmax = FALCON_PRIVKEY_SIZE(FF_FALCON_LOGN);
    size_t pkmax = FALCON_PUBKEY_SIZE(FF_FALCON_LOGN);
    void *tmp = malloc(FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    int r;

    if (tmp == NULL) {
        return -1;
    }

    r = falcon_keygen_make(rng, FF_FALCON_LOGN,
                           sk, skmax,
                           pk, pkmax,
                           tmp, FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    ff_secure_bzero(tmp, FALCON_TMPSIZE_KEYGEN(FF_FALCON_LOGN));
    free(tmp);
    if (r != 0) {
        return -1;
    }

    *sk_len = skmax;
    *pk_len = pkmax;
    return 0;
}

#if defined(NXMS_FALCON_AUDIT_RAW_API) && NXMS_FALCON_AUDIT_RAW_API
static int
ff_falcon_sign_ct_with_rng(shake256_context *rng,
                           const uint8_t *sk, size_t sk_len,
                           const uint8_t *msg, size_t msg_len,
                           uint8_t *sig, size_t *sig_len)
{
    int logn = falcon_get_logn((void *)sk, sk_len);
    size_t tmpsz = FALCON_TMPSIZE_SIGNDYN(FF_FALCON_LOGN);
    void *tmp;
    int r;

    if (logn != FF_FALCON_LOGN) {
        return -1;
    }

    tmp = malloc(tmpsz);
    if (tmp == NULL) {
        return -1;
    }

    r = falcon_sign_dyn(rng,
                        sig, sig_len, FALCON_SIG_CT,
                        sk, sk_len,
                        msg, msg_len,
                        tmp, tmpsz);
    ff_secure_bzero(tmp, tmpsz);
    free(tmp);
    return (r == 0) ? 0 : -1;
}
#endif

static int
ff_falcon_sign_ct_prepared_finish_with_rng(
    shake256_context *rng,
    const ff_falcon_prepared_sk_t *prepared,
    shake256_context *hash_data,
    const uint8_t nonce[40],
    uint8_t *sig, size_t *sig_len)
{
    const unsigned logn = FF_FALCON_LOGN;
    const size_t n = (size_t)1 << logn;
    const size_t tmpsz = FALCON_TMPSIZE_SIGNDYN(logn);
    uint8_t *tmp = malloc(tmpsz);
    inner_shake256_context sav_hash_data;
    uint16_t *hm;
    int16_t *sv;
    uint8_t *atmp;
    unsigned oldcw;
    uint8_t *es;
    size_t es_len;
    size_t v;
    int ret = -1;

    if (!ff_falcon_prepared_is_valid(prepared) || sig == NULL || sig_len == NULL) {
        return -1;
    }
    if (tmp == NULL) {
        return -1;
    }

    es_len = *sig_len;
    if (es_len < FALCON_SIG_CT_SIZE(logn)) {
        goto cleanup;
    }

    hm = (uint16_t *)tmp;
    sv = (int16_t *)hm;
    atmp = ff_align_u64(hm + n);

    shake256_flip(hash_data);
    sav_hash_data = *(inner_shake256_context *)hash_data;
    *(inner_shake256_context *)hash_data = sav_hash_data;
    Zf(hash_to_point_ct)((inner_shake256_context *)hash_data, hm, logn, atmp);

    oldcw = set_fpu_cw(2);
    Zf(sign_dyn)(sv, (inner_shake256_context *)rng,
                 prepared->f, prepared->g, prepared->F, prepared->G,
                 hm, logn, atmp);
    set_fpu_cw(oldcw);

    es = sig;
    es[0] = 0x50 + logn;
    memcpy(es + 1, nonce, 40);
#if defined NXMS_CTGRIND_AUDIT && NXMS_CTGRIND_AUDIT
    /*
     * Signature coefficients become public exactly at the encoding
     * boundary. In the audit build we mark that transition explicitly,
     * so ctgrind does not flag the public range checks in
     * trim_i16_encode().
     */
    VALGRIND_MAKE_MEM_DEFINED(sv, n * sizeof *sv);
#endif
    v = Zf(trim_i16_encode)(es + 41, es_len - 41,
                            sv, logn, Zf(max_sig_bits)[logn]);
    if (v == 0) {
        goto cleanup;
    }

    *sig_len = 41 + v;
    ret = 0;

cleanup:
    ff_secure_bzero(tmp, tmpsz);
    free(tmp);
    return ret;
}

static int
ff_falcon_sign_ct_prepared_with_rng(shake256_context *rng,
                                    const ff_falcon_prepared_sk_t *prepared,
                                    const uint8_t *msg, size_t msg_len,
                                    uint8_t *sig, size_t *sig_len)
{
    uint8_t nonce[40];
    shake256_context hash_data;
    int r;

    if (!ff_falcon_prepared_is_valid(prepared)) {
        return -1;
    }

    r = falcon_sign_start(rng, nonce, &hash_data);
    if (r != 0) {
        return -1;
    }
    if (msg_len > 0) {
        shake256_inject(&hash_data, msg, msg_len);
    }
    return ff_falcon_sign_ct_prepared_finish_with_rng(
        rng, prepared, &hash_data, nonce, sig, sig_len);
}

int
ff_falcon_keygen(uint8_t *sk, size_t *sk_len, uint8_t *pk, size_t *pk_len)
{
    shake256_context rng;
    if (init_rng(&rng) != 0) {
        return -1;
    }
    return ff_falcon_keygen_with_rng(&rng, sk, sk_len, pk, pk_len);
}

int
ff_falcon_keygen_seeded(const uint8_t *seed, size_t seed_len,
                        uint8_t *sk, size_t *sk_len,
                        uint8_t *pk, size_t *pk_len)
{
    shake256_context rng;
    init_rng_from_seed(&rng, seed, seed_len);
    return ff_falcon_keygen_with_rng(&rng, sk, sk_len, pk, pk_len);
}

#if defined(NXMS_FALCON_AUDIT_RAW_API) && NXMS_FALCON_AUDIT_RAW_API
int
ff_falcon_sign_ct(const uint8_t *sk, size_t sk_len,
                  const uint8_t *msg, size_t msg_len,
                  uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;
    if (init_rng(&rng) != 0) {
        return -1;
    }
    return ff_falcon_sign_ct_with_rng(&rng, sk, sk_len, msg, msg_len, sig, sig_len);
}

int
ff_falcon_sign_ct_seeded(const uint8_t *seed, size_t seed_len,
                         const uint8_t *sk, size_t sk_len,
                         const uint8_t *msg, size_t msg_len,
                         uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;
    init_rng_from_seed(&rng, seed, seed_len);
    return ff_falcon_sign_ct_with_rng(&rng, sk, sk_len, msg, msg_len, sig, sig_len);
}
#endif

int
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

    ff_falcon_clear_prepared_sk(prepared);

    if (sk_len != FALCON_PRIVKEY_SIZE(logn) || sk_len == 0) {
        return -1;
    }
    if ((sk[0] & 0xF0) != 0x50 || (sk[0] & 0x0F) != logn) {
        return -1;
    }

    tmp = malloc(FALCON_TMPSIZE_EXPANDPRIV(logn));
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

    prepared->magic = FF_FALCON_PREPARED_MAGIC;
    prepared->logn = logn;
    ff_secure_bzero(tmp, FALCON_TMPSIZE_EXPANDPRIV(logn));
    free(tmp);
    return 0;

fail:
    ff_secure_bzero(tmp, FALCON_TMPSIZE_EXPANDPRIV(logn));
    free(tmp);
    ff_falcon_clear_prepared_sk(prepared);
    return -1;
}

void
ff_falcon_clear_prepared_sk(ff_falcon_prepared_sk_t *prepared)
{
    if (prepared != NULL) {
        ff_secure_bzero(prepared, sizeof *prepared);
    }
}

int
ff_falcon_sign_ct_prepared(const ff_falcon_prepared_sk_t *prepared,
                           const uint8_t *msg, size_t msg_len,
                           uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;
    if (init_rng(&rng) != 0) {
        return -1;
    }
    return ff_falcon_sign_ct_prepared_with_rng(
        &rng, prepared, msg, msg_len, sig, sig_len);
}

int
ff_falcon_sign_ct_prepared_seeded(const uint8_t *seed, size_t seed_len,
                                  const ff_falcon_prepared_sk_t *prepared,
                                  const uint8_t *msg, size_t msg_len,
                                  uint8_t *sig, size_t *sig_len)
{
    shake256_context rng;
    init_rng_from_seed(&rng, seed, seed_len);
    return ff_falcon_sign_ct_prepared_with_rng(
        &rng, prepared, msg, msg_len, sig, sig_len);
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
    if (tmp == NULL) {
        return -1;
    }

    r = falcon_verify(sig, sig_len, FALCON_SIG_CT,
                      pk, pk_len,
                      msg, msg_len,
                      tmp, tmpsz);
    ff_secure_bzero(tmp, tmpsz);
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
    if (ctx != NULL && ctx_len != 0) {
        shake256_inject(&sc, ctx, ctx_len);
    }
    shake256_inject(&sc, in, in_len);
    shake256_flip(&sc);
    shake256_extract(&sc, out, out_len);
}
