#pragma once
#include <stddef.h>
#include <stdint.h>

/*
 * NXMS transport primitives for sealed peer-to-peer messages.
 *
 * Current canonical transport behavior:
 *   - KEM: FrodoKEM-640-SHAKE via ff_kem_* wrappers
 *   - Signature: Falcon-1024-CT via ff_falcon_* wrappers
 *   - AAD/signature inputs are length-prefixed and domain-separated
 *   - Verification order is fail-closed and signature-first:
 *       1) rebuild AAD
 *       2) verify Falcon signature
 *       3) decapsulate Frodo KEM
 *       4) recompute and compare tag
 *       5) decrypt payload
 *
 * IMPORTANT:
 *   The canonical encoding and verification order in this header match the
 *   hardened implementation in nxms_ms_transport.c. Older packet encodings are
 *   not wire-compatible with this implementation.
 */

#ifdef __cplusplus
extern "C" {
#endif

typedef struct nxms_ms_signer_ctx nxms_ms_signer_ctx;

#define NXMS_ESCROW_ID_LEN 16
#define NXMS_NONCE_LEN     24
#define NXMS_TAG_LEN       32

#ifndef NXMS_MAX_PAYLOAD
#define NXMS_MAX_PAYLOAD (16u * 1024u * 1024u)
#endif

#ifndef NXMS_MAX_ID_LEN
#define NXMS_MAX_ID_LEN 128u
#endif

#ifndef NXMS_MIN_KEM_BYTES
#define NXMS_MIN_KEM_BYTES 64u
#endif
#ifndef NXMS_MAX_KEM_PK_LEN
#define NXMS_MAX_KEM_PK_LEN 32768u
#endif
#ifndef NXMS_MAX_KEM_CT_LEN
#define NXMS_MAX_KEM_CT_LEN 32768u
#endif
#ifndef NXMS_MAX_KEM_SK_LEN
#define NXMS_MAX_KEM_SK_LEN 65536u
#endif
#ifndef NXMS_MAX_SIG_SK_LEN
#define NXMS_MAX_SIG_SK_LEN 65536u
#endif
#ifndef NXMS_MAX_SIG_PK_LEN
#define NXMS_MAX_SIG_PK_LEN 65536u
#endif

#define NXMS_KEM_ID "FrodoKEM-640-SHAKE"
#define NXMS_SIG_ID "Falcon-1024-CT"

/*
 * Encrypt + authenticate + sign an application payload.
 *
 * Inputs:
 *   - sender_id / to_id / msg_type are cryptographically bound into AAD
 *   - escrow_id_raw is a 16-byte context identifier bound into KDF and AAD
 *   - seq is a monotonic sequence number per (context, sender)
 *
 * Outputs are heap-allocated and owned by the caller.
 * Free non-secret buffers with nxms_ms_free().
 */
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
                           uint8_t **sig, size_t *sig_len);

/*
 * Prepared signer context for the Falcon-CT sign path.
 *
 * This keeps the same wire format and the same Falcon reference algorithm,
 * but removes secret-key decoding from the per-packet hot path.
 * The context is intended for long-lived signer processes.
 *
 * On supported POSIX systems the implementation attempts best-effort
 * mlock()/munlock() around the prepared secret material; callers should
 * still assume the OS may refuse memory pinning.
 */
nxms_ms_signer_ctx *nxms_ms_signer_ctx_new(const uint8_t *sender_sk_sig,
                                           size_t sender_sk_sig_len);
void nxms_ms_signer_ctx_free(nxms_ms_signer_ctx *ctx);

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
                                       uint8_t **sig, size_t *sig_len);

/*
 * Verify + decrypt.
 *
 * Ownership/contract:
 *   - On entry, if output pointers are provided, function resets them to NULL/0.
 *   - On any error return (rc != 0), outputs remain NULL/0.
 *   - On success (rc == 0), caller owns *out_plain and should wipe it with
 *     nxms_ms_free_secure() if it contains sensitive data.
 */
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
                           uint8_t **out_plain, size_t *out_plain_len);

void nxms_ms_free(void *ptr);
void nxms_ms_free_secure(void *ptr, size_t len);

#ifdef __cplusplus
}
#endif
