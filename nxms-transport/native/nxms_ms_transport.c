#include "nxms_ms_transport.h"

#include "pqc_kem.h"
#include "pqc_falcon.h"
#include "falcon.h"

#include <limits.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#if defined(__unix__) || defined(__APPLE__)
#include <sys/mman.h>
#endif

static const uint8_t AAD_PREFIX[] = "NXMS-AAD-v1";
static const uint8_t SIG_PREFIX[] = "NXMS-SIG-v1";
static const uint8_t CTHASH_PREFIX[] = "NXMS-CTHASH-v1";
static const uint8_t KDF_PREFIX[] = "NXMS-KDF-v1";
static const uint8_t TAG_PREFIX[] = "NXMS-TAG-v1";
static const uint8_t STREAM_PREFIX[] = "NXMS-STREAM-v1";

#define NXMS_MS_SIGNER_MAGIC 0x4E584D53u

struct nxms_ms_signer_ctx {
    uint32_t magic;
    int is_mlocked;
    ff_falcon_prepared_sk_t prepared;
};

static void memzero(void *p, size_t n) {
    volatile uint8_t *vp = (volatile uint8_t *)p;
    while (n--) {
        *vp++ = 0;
    }
}

static void secure_free(void *p, size_t n) {
    if (p) {
        memzero(p, n);
        free(p);
    }
}

static int nxms_ms_signer_ctx_is_valid(const nxms_ms_signer_ctx *ctx) {
    return ctx != NULL && ctx->magic == NXMS_MS_SIGNER_MAGIC;
}

static void nxms_ms_reset_encrypt_outputs(uint8_t **kem_ct, size_t *kem_ct_len,
                                          uint8_t **nonce, size_t *nonce_len,
                                          uint8_t **ciphertext, size_t *ciphertext_len,
                                          uint8_t **tag, size_t *tag_len,
                                          uint8_t **sig, size_t *sig_len) {
    *kem_ct = NULL;
    *kem_ct_len = 0;
    *nonce = NULL;
    *nonce_len = 0;
    *ciphertext = NULL;
    *ciphertext_len = 0;
    *tag = NULL;
    *tag_len = 0;
    *sig = NULL;
    *sig_len = 0;
}

static int nxms_ms_sign_message(const nxms_ms_signer_ctx *signer,
                                const uint8_t *sender_sk_sig,
                                size_t sender_sk_sig_len,
                                const uint8_t *sig_msg,
                                size_t sig_msg_len,
                                uint8_t *sig_out,
                                size_t *sig_out_len) {
    if (signer != NULL) {
        if (!nxms_ms_signer_ctx_is_valid(signer)) {
            return -1;
        }
        return ff_falcon_sign_ct_prepared(&signer->prepared,
                                          sig_msg, sig_msg_len,
                                          sig_out, sig_out_len);
    }
    if (sender_sk_sig == NULL || sender_sk_sig_len == 0) {
        return -1;
    }
    return ff_falcon_sign_ct(sender_sk_sig, sender_sk_sig_len,
                             sig_msg, sig_msg_len,
                             sig_out, sig_out_len);
}

static int add_sz(size_t *acc, size_t x) {
    if (!acc) {
        return -1;
    }
    if (*acc > SIZE_MAX - x) {
        return -1;
    }
    *acc += x;
    return 0;
}

static size_t nxms_strnlen(const char *s, size_t max) {
    size_t n = 0;
    if (!s) {
        return 0;
    }
    while (n < max && s[n] != 0) {
        n++;
    }
    return n;
}

static void u32be(uint32_t x, uint8_t out[4]) {
    out[0] = (uint8_t)(x >> 24);
    out[1] = (uint8_t)(x >> 16);
    out[2] = (uint8_t)(x >> 8);
    out[3] = (uint8_t)x;
}

static void u64be(uint64_t x, uint8_t out[8]) {
    out[0] = (uint8_t)(x >> 56);
    out[1] = (uint8_t)(x >> 48);
    out[2] = (uint8_t)(x >> 40);
    out[3] = (uint8_t)(x >> 32);
    out[4] = (uint8_t)(x >> 24);
    out[5] = (uint8_t)(x >> 16);
    out[6] = (uint8_t)(x >> 8);
    out[7] = (uint8_t)x;
}

static int rand_bytes(uint8_t *out, size_t out_len) {
    shake256_context rng;

    if (!out && out_len) {
        return -1;
    }
    if (shake256_init_prng_from_system(&rng) != 0) {
        memzero(&rng, sizeof(rng));
        return -1;
    }
    shake256_flip(&rng);
    shake256_extract(&rng, out, out_len);
    memzero(&rng, sizeof(rng));
    return 0;
}

static void xor_bytes(uint8_t *a, const uint8_t *b, size_t n) {
    size_t i;
    for (i = 0; i < n; i++) {
        a[i] ^= b[i];
    }
}

static int ct_memeq(const uint8_t *a, const uint8_t *b, size_t n) {
    uint8_t diff = 0;
    size_t i;

    for (i = 0; i < n; i++) {
        diff |= (uint8_t)(a[i] ^ b[i]);
    }
    return diff == 0 ? 0 : -1;
}

static int write_u32_and_bytes(uint8_t **p, const uint8_t *b, size_t n) {
    uint8_t be[4];

    if (!p) {
        return -1;
    }
    if (n > UINT32_MAX) {
        return -1;
    }
    u32be((uint32_t)n, be);
    memcpy(*p, be, sizeof(be));
    *p += sizeof(be);
    if (n) {
        memcpy(*p, b, n);
        *p += n;
    }
    return 0;
}

/*
 * XOR-in-place keystream:
 *   SHAKE256("NXMS-STREAM-v1" || ke || nonce)
 */
static int xor_shake_keystream(uint8_t *buf, size_t len,
                               const uint8_t ke[32],
                               const uint8_t nonce[NXMS_NONCE_LEN]) {
    shake256_context sc;
    uint8_t block[4096];
    size_t off = 0;

    if (!buf || len == 0) {
        return 0;
    }
    if (!ke || !nonce) {
        return -1;
    }

    shake256_init(&sc);
    shake256_inject(&sc, STREAM_PREFIX, sizeof(STREAM_PREFIX) - 1);
    shake256_inject(&sc, ke, 32);
    shake256_inject(&sc, nonce, NXMS_NONCE_LEN);
    shake256_flip(&sc);

    while (off < len) {
        size_t n = len - off;
        if (n > sizeof(block)) {
            n = sizeof(block);
        }
        shake256_extract(&sc, block, n);
        xor_bytes(buf + off, block, n);
        off += n;
    }

    memzero(block, sizeof(block));
    memzero(&sc, sizeof(sc));
    return 0;
}

/*
 * ct_hash32 = SHAKE256("NXMS-CTHASH-v1" || ct)[0..32)
 */
static void shake_hash32(uint8_t out32[32], const uint8_t *ct, size_t ct_len) {
    shake256_context sc;

    shake256_init(&sc);
    shake256_inject(&sc, CTHASH_PREFIX, sizeof(CTHASH_PREFIX) - 1);
    shake256_inject(&sc, ct, ct_len);
    shake256_flip(&sc);
    shake256_extract(&sc, out32, 32);
    memzero(&sc, sizeof(sc));
}

/*
 * AAD (length-prefixed, unambiguous):
 *   AAD_PREFIX
 *   u32(sender_len) sender
 *   u32(to_len)     to
 *   u32(kem_len)    NXMS_KEM_ID
 *   u32(sig_len)    NXMS_SIG_ID
 *   u32(msg_len)    msg_type
 *   escrow_id_raw (16)
 *   seq_be (8)
 *   ct_hash32 (32)
 */
static int build_aad(const char *sender_id, const char *to_id, const char *msg_type,
                     const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                     uint64_t seq,
                     const uint8_t *ct, size_t ct_len,
                     uint8_t **out, size_t *out_len) {
    const size_t kem_len = strlen(NXMS_KEM_ID);
    const size_t sig_len = strlen(NXMS_SIG_ID);
    size_t sender_len;
    size_t to_len;
    size_t msg_len;
    uint8_t ct_hash[32];
    uint8_t seq_be[8];
    size_t len = 0;
    uint8_t *buf;
    uint8_t *p;

    if (!sender_id || !to_id || !msg_type || !escrow_id_raw || !ct || !out || !out_len) {
        return -1;
    }
    if (ct_len < NXMS_MIN_KEM_BYTES || ct_len > NXMS_MAX_KEM_CT_LEN) {
        return -1;
    }
    if (seq == 0) {
        return -1;
    }

    sender_len = nxms_strnlen(sender_id, NXMS_MAX_ID_LEN + 1);
    to_len = nxms_strnlen(to_id, NXMS_MAX_ID_LEN + 1);
    msg_len = nxms_strnlen(msg_type, NXMS_MAX_ID_LEN + 1);
    if (sender_len == 0 || to_len == 0 || msg_len == 0) {
        return -1;
    }
    if (sender_len > NXMS_MAX_ID_LEN || to_len > NXMS_MAX_ID_LEN || msg_len > NXMS_MAX_ID_LEN) {
        return -1;
    }

    shake_hash32(ct_hash, ct, ct_len);
    u64be(seq, seq_be);

    if (add_sz(&len, sizeof(AAD_PREFIX) - 1) != 0 ||
        add_sz(&len, 4 + sender_len) != 0 ||
        add_sz(&len, 4 + to_len) != 0 ||
        add_sz(&len, 4 + kem_len) != 0 ||
        add_sz(&len, 4 + sig_len) != 0 ||
        add_sz(&len, 4 + msg_len) != 0 ||
        add_sz(&len, NXMS_ESCROW_ID_LEN) != 0 ||
        add_sz(&len, sizeof(seq_be)) != 0 ||
        add_sz(&len, sizeof(ct_hash)) != 0) {
        memzero(ct_hash, sizeof(ct_hash));
        return -1;
    }

    buf = (uint8_t *)malloc(len ? len : 1);
    if (!buf) {
        memzero(ct_hash, sizeof(ct_hash));
        return -1;
    }

    p = buf;
    memcpy(p, AAD_PREFIX, sizeof(AAD_PREFIX) - 1);
    p += sizeof(AAD_PREFIX) - 1;
    if (write_u32_and_bytes(&p, (const uint8_t *)sender_id, sender_len) != 0 ||
        write_u32_and_bytes(&p, (const uint8_t *)to_id, to_len) != 0 ||
        write_u32_and_bytes(&p, (const uint8_t *)NXMS_KEM_ID, kem_len) != 0 ||
        write_u32_and_bytes(&p, (const uint8_t *)NXMS_SIG_ID, sig_len) != 0 ||
        write_u32_and_bytes(&p, (const uint8_t *)msg_type, msg_len) != 0) {
        memzero(ct_hash, sizeof(ct_hash));
        free(buf);
        return -1;
    }

    memcpy(p, escrow_id_raw, NXMS_ESCROW_ID_LEN);
    p += NXMS_ESCROW_ID_LEN;
    memcpy(p, seq_be, sizeof(seq_be));
    p += sizeof(seq_be);
    memcpy(p, ct_hash, sizeof(ct_hash));

    memzero(ct_hash, sizeof(ct_hash));
    *out = buf;
    *out_len = len;
    return 0;
}

/*
 * KDF (domain separated; no temp allocation):
 *   ke = SHAKE256("NXMS-KDF-v1" || u32(ss_len)||ss || escrow_id || "ms-ke")[0..32)
 *   km = SHAKE256("NXMS-KDF-v1" || u32(ss_len)||ss || escrow_id || "ms-km")[0..32)
 */
static int derive_keys(const uint8_t *ss, size_t ss_len,
                       const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                       uint8_t ke[32], uint8_t km[32]) {
    static const uint8_t LBL_KE[] = "ms-ke";
    static const uint8_t LBL_KM[] = "ms-km";
    uint8_t ss_len_be[4];
    shake256_context sc;

    if (!ss || ss_len == 0 || !escrow_id_raw || !ke || !km) {
        return -1;
    }
    if (ss_len > UINT32_MAX) {
        return -1;
    }

    u32be((uint32_t)ss_len, ss_len_be);

    shake256_init(&sc);
    shake256_inject(&sc, KDF_PREFIX, sizeof(KDF_PREFIX) - 1);
    shake256_inject(&sc, ss_len_be, sizeof(ss_len_be));
    shake256_inject(&sc, ss, ss_len);
    shake256_inject(&sc, escrow_id_raw, NXMS_ESCROW_ID_LEN);
    shake256_inject(&sc, LBL_KE, sizeof(LBL_KE) - 1);
    shake256_flip(&sc);
    shake256_extract(&sc, ke, 32);
    memzero(&sc, sizeof(sc));

    shake256_init(&sc);
    shake256_inject(&sc, KDF_PREFIX, sizeof(KDF_PREFIX) - 1);
    shake256_inject(&sc, ss_len_be, sizeof(ss_len_be));
    shake256_inject(&sc, ss, ss_len);
    shake256_inject(&sc, escrow_id_raw, NXMS_ESCROW_ID_LEN);
    shake256_inject(&sc, LBL_KM, sizeof(LBL_KM) - 1);
    shake256_flip(&sc);
    shake256_extract(&sc, km, 32);
    memzero(&sc, sizeof(sc));

    return 0;
}

/*
 * tag = SHAKE256("NXMS-TAG-v1" || km ||
 *                u32(aad_len)||aad ||
 *                u32(nonce_len)||nonce ||
 *                u32(ct_len)||ciphertext)[0..NXMS_TAG_LEN)
 */
static int compute_tag(uint8_t out32[NXMS_TAG_LEN],
                       const uint8_t km[32],
                       const uint8_t *aad, size_t aad_len,
                       const uint8_t *nonce, size_t nonce_len,
                       const uint8_t *ciphertext, size_t ciphertext_len) {
    uint8_t aad_len_be[4];
    uint8_t nonce_len_be[4];
    uint8_t ct_len_be[4];
    shake256_context sc;

    if (!out32 || !km || !aad || !nonce) {
        return -1;
    }
    if (nonce_len != NXMS_NONCE_LEN) {
        return -1;
    }
    if (aad_len > UINT32_MAX || ciphertext_len > UINT32_MAX) {
        return -1;
    }
    if (ciphertext_len > 0 && !ciphertext) {
        return -1;
    }

    u32be((uint32_t)aad_len, aad_len_be);
    u32be((uint32_t)nonce_len, nonce_len_be);
    u32be((uint32_t)ciphertext_len, ct_len_be);

    shake256_init(&sc);
    shake256_inject(&sc, TAG_PREFIX, sizeof(TAG_PREFIX) - 1);
    shake256_inject(&sc, km, 32);
    shake256_inject(&sc, aad_len_be, sizeof(aad_len_be));
    shake256_inject(&sc, aad, aad_len);
    shake256_inject(&sc, nonce_len_be, sizeof(nonce_len_be));
    shake256_inject(&sc, nonce, nonce_len);
    shake256_inject(&sc, ct_len_be, sizeof(ct_len_be));
    if (ciphertext_len) {
        shake256_inject(&sc, ciphertext, ciphertext_len);
    }
    shake256_flip(&sc);
    shake256_extract(&sc, out32, NXMS_TAG_LEN);
    memzero(&sc, sizeof(sc));
    return 0;
}

/*
 * Signature message (length-prefixed, unambiguous):
 *   SIG_PREFIX
 *   u32(aad_len)   aad
 *   u32(nonce_len) nonce
 *   u32(ct_len)    ciphertext
 *   u32(tag_len)   tag
 */
static int build_sig_message(const uint8_t *aad, size_t aad_len,
                             const uint8_t *nonce, size_t nonce_len,
                             const uint8_t *ciphertext, size_t ciphertext_len,
                             const uint8_t tag32[NXMS_TAG_LEN],
                             uint8_t **out, size_t *out_len) {
    size_t len = 0;
    uint8_t *buf;
    uint8_t *p;

    if (!aad || !nonce || !tag32 || !out || !out_len) {
        return -1;
    }
    if (nonce_len != NXMS_NONCE_LEN) {
        return -1;
    }
    if (aad_len > UINT32_MAX || ciphertext_len > UINT32_MAX) {
        return -1;
    }
    if (ciphertext_len > 0 && !ciphertext) {
        return -1;
    }

    if (add_sz(&len, sizeof(SIG_PREFIX) - 1) != 0 ||
        add_sz(&len, 4 + aad_len) != 0 ||
        add_sz(&len, 4 + nonce_len) != 0 ||
        add_sz(&len, 4 + ciphertext_len) != 0 ||
        add_sz(&len, 4 + NXMS_TAG_LEN) != 0) {
        return -1;
    }

    buf = (uint8_t *)malloc(len ? len : 1);
    if (!buf) {
        return -1;
    }

    p = buf;
    memcpy(p, SIG_PREFIX, sizeof(SIG_PREFIX) - 1);
    p += sizeof(SIG_PREFIX) - 1;
    if (write_u32_and_bytes(&p, aad, aad_len) != 0 ||
        write_u32_and_bytes(&p, nonce, nonce_len) != 0 ||
        write_u32_and_bytes(&p, ciphertext, ciphertext_len) != 0 ||
        write_u32_and_bytes(&p, tag32, NXMS_TAG_LEN) != 0) {
        free(buf);
        return -1;
    }

    *out = buf;
    *out_len = len;
    return 0;
}

nxms_ms_signer_ctx *nxms_ms_signer_ctx_new(const uint8_t *sender_sk_sig,
                                           size_t sender_sk_sig_len) {
    nxms_ms_signer_ctx *ctx;

    if (sender_sk_sig == NULL || sender_sk_sig_len == 0 ||
        sender_sk_sig_len > NXMS_MAX_SIG_SK_LEN) {
        return NULL;
    }

    ctx = (nxms_ms_signer_ctx *)calloc(1, sizeof(*ctx));
    if (ctx == NULL) {
        return NULL;
    }

#if defined(__unix__) || defined(__APPLE__)
    if (mlock(ctx, sizeof(*ctx)) == 0) {
        ctx->is_mlocked = 1;
    }
#endif

    if (ff_falcon_prepare_sk(sender_sk_sig, sender_sk_sig_len, &ctx->prepared) != 0) {
        nxms_ms_signer_ctx_free(ctx);
        return NULL;
    }

    ctx->magic = NXMS_MS_SIGNER_MAGIC;
    return ctx;
}

void nxms_ms_signer_ctx_free(nxms_ms_signer_ctx *ctx) {
    if (ctx == NULL) {
        return;
    }
    ff_falcon_clear_prepared_sk(&ctx->prepared);
    ctx->magic = 0;
#if defined(__unix__) || defined(__APPLE__)
    if (ctx->is_mlocked) {
        memzero(ctx, sizeof(*ctx));
        munlock(ctx, sizeof(*ctx));
        free(ctx);
        return;
    }
#endif
    memzero(ctx, sizeof(*ctx));
    free(ctx);
}

static int nxms_ms_encrypt_packet_impl(const char *sender_id,
                                       const char *to_id,
                                       const char *msg_type,
                                       const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                                       uint64_t seq,
                                       const uint8_t *recipient_pk_kem, size_t recipient_pk_kem_len,
                                       const uint8_t *sender_sk_sig, size_t sender_sk_sig_len,
                                       const nxms_ms_signer_ctx *signer,
                                       const uint8_t *plaintext, size_t plaintext_len,
                                       uint8_t **kem_ct, size_t *kem_ct_len,
                                       uint8_t **nonce, size_t *nonce_len,
                                       uint8_t **ciphertext, size_t *ciphertext_len,
                                       uint8_t **tag, size_t *tag_len,
                                       uint8_t **sig, size_t *sig_len) {
    uint8_t *ct_loc = NULL;
    uint8_t *ss = NULL;
    size_t ct_loc_len = 0;
    size_t ss_len = 0;
    uint8_t ke[32];
    uint8_t km[32];
    uint8_t *nonce_loc = NULL;
    uint8_t *ctext_loc = NULL;
    uint8_t *aad = NULL;
    size_t aad_len = 0;
    uint8_t tag32[NXMS_TAG_LEN];
    uint8_t *tag_loc = NULL;
    uint8_t *sig_msg = NULL;
    size_t sig_msg_len = 0;
    uint8_t *sig_loc = NULL;
    size_t sig_loc_len = FF_FALCON_SIG_MAX;
    size_t sid_len;
    size_t tid_len;
    size_t mt_len;

    if (!kem_ct || !kem_ct_len || !nonce || !nonce_len ||
        !ciphertext || !ciphertext_len || !tag || !tag_len || !sig || !sig_len) {
        return -1;
    }

    nxms_ms_reset_encrypt_outputs(
        kem_ct, kem_ct_len, nonce, nonce_len, ciphertext, ciphertext_len, tag, tag_len, sig,
        sig_len);

    if (!sender_id || !to_id || !msg_type || !escrow_id_raw ||
        !recipient_pk_kem) {
        return -1;
    }
    if (seq == 0) {
        return -1;
    }
    if (plaintext_len > NXMS_MAX_PAYLOAD) {
        return -1;
    }
    if (plaintext_len > 0 && !plaintext) {
        return -1;
    }
    if (recipient_pk_kem_len < NXMS_MIN_KEM_BYTES || recipient_pk_kem_len > NXMS_MAX_KEM_PK_LEN) {
        return -1;
    }
    if (signer == NULL &&
        (sender_sk_sig == NULL || sender_sk_sig_len == 0 ||
         sender_sk_sig_len > NXMS_MAX_SIG_SK_LEN)) {
        return -1;
    }
    if (signer != NULL && !nxms_ms_signer_ctx_is_valid(signer)) {
        return -1;
    }

    sid_len = nxms_strnlen(sender_id, NXMS_MAX_ID_LEN + 1);
    tid_len = nxms_strnlen(to_id, NXMS_MAX_ID_LEN + 1);
    mt_len = nxms_strnlen(msg_type, NXMS_MAX_ID_LEN + 1);
    if (sid_len == 0 || tid_len == 0 || mt_len == 0) {
        return -1;
    }
    if (sid_len > NXMS_MAX_ID_LEN || tid_len > NXMS_MAX_ID_LEN || mt_len > NXMS_MAX_ID_LEN) {
        return -1;
    }

    if (ff_kem_encaps(NXMS_KEM_ID, recipient_pk_kem, recipient_pk_kem_len,
                      &ct_loc, &ct_loc_len, &ss, &ss_len) != 0) {
        return -1;
    }
    if (ct_loc_len < NXMS_MIN_KEM_BYTES || ct_loc_len > NXMS_MAX_KEM_CT_LEN || ss_len == 0) {
        secure_free(ss, ss_len);
        free(ct_loc);
        return -1;
    }

    if (derive_keys(ss, ss_len, escrow_id_raw, ke, km) != 0) {
        secure_free(ss, ss_len);
        free(ct_loc);
        return -1;
    }
    secure_free(ss, ss_len);

    nonce_loc = (uint8_t *)malloc(NXMS_NONCE_LEN);
    if (!nonce_loc) {
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }
    if (rand_bytes(nonce_loc, NXMS_NONCE_LEN) != 0) {
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    ctext_loc = (uint8_t *)malloc(plaintext_len ? plaintext_len : 1);
    if (!ctext_loc) {
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }
    if (plaintext_len) {
        memcpy(ctext_loc, plaintext, plaintext_len);
        if (xor_shake_keystream(ctext_loc, plaintext_len, ke, nonce_loc) != 0) {
            secure_free(ctext_loc, plaintext_len);
            free(nonce_loc);
            free(ct_loc);
            memzero(ke, sizeof(ke));
            memzero(km, sizeof(km));
            return -1;
        }
    }

    if (build_aad(sender_id, to_id, msg_type, escrow_id_raw, seq,
                  ct_loc, ct_loc_len, &aad, &aad_len) != 0) {
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    if (compute_tag(tag32, km, aad, aad_len, nonce_loc, NXMS_NONCE_LEN,
                    ctext_loc, plaintext_len) != 0) {
        free(aad);
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    tag_loc = (uint8_t *)malloc(NXMS_TAG_LEN);
    if (!tag_loc) {
        memzero(tag32, sizeof(tag32));
        free(aad);
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }
    memcpy(tag_loc, tag32, NXMS_TAG_LEN);
    memzero(tag32, sizeof(tag32));

    if (build_sig_message(aad, aad_len, nonce_loc, NXMS_NONCE_LEN,
                          ctext_loc, plaintext_len, tag_loc,
                          &sig_msg, &sig_msg_len) != 0) {
        free(tag_loc);
        free(aad);
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    sig_loc = (uint8_t *)malloc(FF_FALCON_SIG_MAX);
    if (!sig_loc) {
        free(sig_msg);
        free(tag_loc);
        free(aad);
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    if (nxms_ms_sign_message(signer, sender_sk_sig, sender_sk_sig_len,
                             sig_msg, sig_msg_len,
                             sig_loc, &sig_loc_len) != 0) {
        free(sig_loc);
        free(sig_msg);
        free(tag_loc);
        free(aad);
        secure_free(ctext_loc, plaintext_len);
        free(nonce_loc);
        free(ct_loc);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    free(sig_msg);
    free(aad);
    memzero(ke, sizeof(ke));
    memzero(km, sizeof(km));

    *kem_ct = ct_loc;
    *kem_ct_len = ct_loc_len;
    *nonce = nonce_loc;
    *nonce_len = NXMS_NONCE_LEN;
    *ciphertext = ctext_loc;
    *ciphertext_len = plaintext_len;
    *tag = tag_loc;
    *tag_len = NXMS_TAG_LEN;
    *sig = sig_loc;
    *sig_len = sig_loc_len;
    return 0;
}

int nxms_ms_encrypt_packet(const char *sender_id,
                           const char *to_id,
                           const char *msg_type,
                           const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                           uint64_t seq,
                           const uint8_t *recipient_pk_kem, size_t recipient_pk_kem_len,
                           const uint8_t *sender_sk_sig, size_t sender_sk_sig_len,
                           const uint8_t *plaintext, size_t plaintext_len,
                           uint8_t **kem_ct, size_t *kem_ct_len,
                           uint8_t **nonce, size_t *nonce_len,
                           uint8_t **ciphertext, size_t *ciphertext_len,
                           uint8_t **tag, size_t *tag_len,
                           uint8_t **sig, size_t *sig_len) {
    return nxms_ms_encrypt_packet_impl(
        sender_id, to_id, msg_type, escrow_id_raw, seq, recipient_pk_kem, recipient_pk_kem_len,
        sender_sk_sig, sender_sk_sig_len, NULL, plaintext, plaintext_len, kem_ct, kem_ct_len,
        nonce, nonce_len, ciphertext, ciphertext_len, tag, tag_len, sig, sig_len);
}

int nxms_ms_encrypt_packet_with_signer(const char *sender_id,
                                       const char *to_id,
                                       const char *msg_type,
                                       const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                                       uint64_t seq,
                                       const uint8_t *recipient_pk_kem, size_t recipient_pk_kem_len,
                                       const nxms_ms_signer_ctx *signer,
                                       const uint8_t *plaintext, size_t plaintext_len,
                                       uint8_t **kem_ct, size_t *kem_ct_len,
                                       uint8_t **nonce, size_t *nonce_len,
                                       uint8_t **ciphertext, size_t *ciphertext_len,
                                       uint8_t **tag, size_t *tag_len,
                                       uint8_t **sig, size_t *sig_len) {
    return nxms_ms_encrypt_packet_impl(
        sender_id, to_id, msg_type, escrow_id_raw, seq, recipient_pk_kem, recipient_pk_kem_len,
        NULL, 0, signer, plaintext, plaintext_len, kem_ct, kem_ct_len, nonce, nonce_len,
        ciphertext, ciphertext_len, tag, tag_len, sig, sig_len);
}

int nxms_ms_verify_decrypt(const char *sender_id,
                           const char *to_id,
                           const char *msg_type,
                           const uint8_t escrow_id_raw[NXMS_ESCROW_ID_LEN],
                           uint64_t seq,
                           const uint8_t *kem_ct, size_t kem_ct_len,
                           const uint8_t *nonce, size_t nonce_len,
                           const uint8_t *ciphertext, size_t ciphertext_len,
                           const uint8_t *tag, size_t tag_len,
                           const uint8_t *sig, size_t sig_len,
                           const uint8_t *recipient_sk_kem, size_t recipient_sk_kem_len,
                           const uint8_t *sender_pk_sig, size_t sender_pk_sig_len,
                           uint8_t **out_plain, size_t *out_plain_len) {
    uint8_t *aad = NULL;
    size_t aad_len = 0;
    uint8_t *sig_msg = NULL;
    size_t sig_msg_len = 0;
    uint8_t *ss = NULL;
    size_t ss_len = 0;
    uint8_t ke[32];
    uint8_t km[32];
    uint8_t tag_exp[NXMS_TAG_LEN];
    int tag_ok;
    uint8_t *plain;
    size_t sid_len;
    size_t tid_len;
    size_t mt_len;

    if (!out_plain || !out_plain_len) {
        return -1;
    }
    *out_plain = NULL;
    *out_plain_len = 0;

    if (!sender_id || !to_id || !msg_type || !escrow_id_raw ||
        !kem_ct || !nonce || !tag || !sig || !recipient_sk_kem || !sender_pk_sig) {
        return -1;
    }
    if (seq == 0) {
        return -1;
    }
    if (nonce_len != NXMS_NONCE_LEN || tag_len != NXMS_TAG_LEN) {
        return -1;
    }
    if (ciphertext_len > NXMS_MAX_PAYLOAD) {
        return -1;
    }
    if (ciphertext_len > 0 && !ciphertext) {
        return -1;
    }
    if (kem_ct_len < NXMS_MIN_KEM_BYTES || kem_ct_len > NXMS_MAX_KEM_CT_LEN) {
        return -1;
    }
    if (sig_len == 0 || sig_len > FF_FALCON_SIG_MAX) {
        return -1;
    }
    if (recipient_sk_kem_len < NXMS_MIN_KEM_BYTES || recipient_sk_kem_len > NXMS_MAX_KEM_SK_LEN) {
        return -1;
    }
    if (sender_pk_sig_len == 0 || sender_pk_sig_len > NXMS_MAX_SIG_PK_LEN) {
        return -1;
    }

    sid_len = nxms_strnlen(sender_id, NXMS_MAX_ID_LEN + 1);
    tid_len = nxms_strnlen(to_id, NXMS_MAX_ID_LEN + 1);
    mt_len = nxms_strnlen(msg_type, NXMS_MAX_ID_LEN + 1);
    if (sid_len == 0 || tid_len == 0 || mt_len == 0) {
        return -1;
    }
    if (sid_len > NXMS_MAX_ID_LEN || tid_len > NXMS_MAX_ID_LEN || mt_len > NXMS_MAX_ID_LEN) {
        return -1;
    }

    /*
     * Verify signature before KEM decapsulation to avoid spending work on
     * unauthenticated input.
     */
    if (build_aad(sender_id, to_id, msg_type, escrow_id_raw, seq,
                  kem_ct, kem_ct_len, &aad, &aad_len) != 0) {
        return -1;
    }
    if (build_sig_message(aad, aad_len, nonce, nonce_len, ciphertext, ciphertext_len,
                          tag, &sig_msg, &sig_msg_len) != 0) {
        free(aad);
        return -1;
    }
    if (ff_falcon_verify(sender_pk_sig, sender_pk_sig_len,
                         sig_msg, sig_msg_len, sig, sig_len) != 0) {
        free(sig_msg);
        free(aad);
        return -1;
    }
    free(sig_msg);

    if (ff_kem_decaps(NXMS_KEM_ID, recipient_sk_kem, recipient_sk_kem_len,
                      kem_ct, kem_ct_len, &ss, &ss_len) != 0) {
        free(aad);
        return -1;
    }
    if (ss_len == 0) {
        secure_free(ss, ss_len);
        free(aad);
        return -1;
    }
    if (derive_keys(ss, ss_len, escrow_id_raw, ke, km) != 0) {
        secure_free(ss, ss_len);
        free(aad);
        return -1;
    }
    secure_free(ss, ss_len);

    if (compute_tag(tag_exp, km, aad, aad_len, nonce, nonce_len,
                    ciphertext, ciphertext_len) != 0) {
        free(aad);
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }
    tag_ok = ct_memeq(tag_exp, tag, NXMS_TAG_LEN);
    memzero(tag_exp, sizeof(tag_exp));
    free(aad);
    if (tag_ok != 0) {
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }

    plain = (uint8_t *)malloc(ciphertext_len ? ciphertext_len : 1);
    if (!plain) {
        memzero(ke, sizeof(ke));
        memzero(km, sizeof(km));
        return -1;
    }
    if (ciphertext_len) {
        memcpy(plain, ciphertext, ciphertext_len);
        if (xor_shake_keystream(plain, ciphertext_len, ke, nonce) != 0) {
            secure_free(plain, ciphertext_len);
            memzero(ke, sizeof(ke));
            memzero(km, sizeof(km));
            return -1;
        }
    }

    memzero(ke, sizeof(ke));
    memzero(km, sizeof(km));

    *out_plain = plain;
    *out_plain_len = ciphertext_len;
    return 0;
}

void nxms_ms_free(void *ptr) {
    free(ptr);
}

void nxms_ms_free_secure(void *ptr, size_t len) {
    secure_free(ptr, len);
}
