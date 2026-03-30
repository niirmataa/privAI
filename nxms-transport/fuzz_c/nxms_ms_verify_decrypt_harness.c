#include "nxms_ms_transport.h"
#include "pqc_falcon.h"
#include "pqc_kem.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FIXTURE_MAGIC "NXMSFIX1"
#define INPUT_MAGIC   "NXMSINP1"

typedef struct {
    uint8_t *recipient_sk_kem;
    size_t recipient_sk_kem_len;
    uint8_t *sender_pk_sig;
    size_t sender_pk_sig_len;
} fixture_t;

typedef struct {
    char *sender_id;
    char *to_id;
    char *msg_type;
    uint8_t escrow_id[NXMS_ESCROW_ID_LEN];
    uint64_t seq;
    const uint8_t *kem_ct;
    size_t kem_ct_len;
    const uint8_t *nonce;
    size_t nonce_len;
    const uint8_t *ciphertext;
    size_t ciphertext_len;
    const uint8_t *tag;
    size_t tag_len;
    const uint8_t *sig;
    size_t sig_len;
} fuzz_case_t;

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

static uint16_t read_u16_le(const uint8_t *p) {
    return (uint16_t)p[0] | ((uint16_t)p[1] << 8);
}

static uint32_t read_u32_le(const uint8_t *p) {
    return (uint32_t)p[0]
        | ((uint32_t)p[1] << 8)
        | ((uint32_t)p[2] << 16)
        | ((uint32_t)p[3] << 24);
}

static uint64_t read_u64_le(const uint8_t *p) {
    uint64_t v = 0;
    size_t i;
    for (i = 0; i < 8; i++) {
        v |= ((uint64_t)p[i]) << (8u * i);
    }
    return v;
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

static int dup_c_string(const uint8_t *src, size_t len, char **out) {
    char *s;
    if (!out) {
        return -1;
    }
    *out = NULL;
    if (len > NXMS_MAX_ID_LEN) {
        return -1;
    }
    s = (char *)malloc(len + 1u);
    if (!s) {
        return -1;
    }
    if (len) {
        memcpy(s, src, len);
    }
    s[len] = '\0';
    *out = s;
    return 0;
}

static void fixture_free(fixture_t *fixture) {
    if (!fixture) {
        return;
    }
    secure_free(fixture->recipient_sk_kem, fixture->recipient_sk_kem_len);
    fixture->recipient_sk_kem = NULL;
    fixture->recipient_sk_kem_len = 0;
    secure_free(fixture->sender_pk_sig, fixture->sender_pk_sig_len);
    fixture->sender_pk_sig = NULL;
    fixture->sender_pk_sig_len = 0;
}

static void fuzz_case_free(fuzz_case_t *fc) {
    if (!fc) {
        return;
    }
    free(fc->sender_id);
    fc->sender_id = NULL;
    free(fc->to_id);
    fc->to_id = NULL;
    free(fc->msg_type);
    fc->msg_type = NULL;
}

static int load_fixture(const char *path, fixture_t *out) {
    uint8_t *buf = NULL;
    size_t len = 0;
    size_t off = 0;
    uint32_t sk_len, pk_len;

    if (!path || !out) {
        return -1;
    }
    memset(out, 0, sizeof(*out));
    buf = read_file(path, &len);
    if (!buf) {
        return -1;
    }
    if (len < 8 + 4 + 4) {
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
    out->recipient_sk_kem = (uint8_t *)malloc(sk_len ? sk_len : 1u);
    if (!out->recipient_sk_kem) {
        free(buf);
        return -1;
    }
    memcpy(out->recipient_sk_kem, buf + off, sk_len);
    out->recipient_sk_kem_len = sk_len;
    off += sk_len;

    pk_len = read_u32_le(buf + off);
    off += 4;
    if (off + pk_len != len) {
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
    free(buf);
    return 0;
}

static int parse_input(const uint8_t *buf, size_t len, fuzz_case_t *out) {
    size_t off = 0;
    uint16_t sender_len, to_len, msg_len;
    uint16_t kem_ct_len, nonce_len, ciphertext_len, tag_len, sig_len;
    size_t needed;

    if (!buf || !out || len < 8 + 8 + NXMS_ESCROW_ID_LEN + 16) {
        return -1;
    }
    memset(out, 0, sizeof(*out));
    if (memcmp(buf, INPUT_MAGIC, 8) != 0) {
        return -1;
    }
    off = 8;
    out->seq = read_u64_le(buf + off);
    off += 8;
    memcpy(out->escrow_id, buf + off, NXMS_ESCROW_ID_LEN);
    off += NXMS_ESCROW_ID_LEN;

    sender_len = read_u16_le(buf + off); off += 2;
    to_len = read_u16_le(buf + off); off += 2;
    msg_len = read_u16_le(buf + off); off += 2;
    kem_ct_len = read_u16_le(buf + off); off += 2;
    nonce_len = read_u16_le(buf + off); off += 2;
    ciphertext_len = read_u16_le(buf + off); off += 2;
    tag_len = read_u16_le(buf + off); off += 2;
    sig_len = read_u16_le(buf + off); off += 2;

    needed = (size_t)sender_len + (size_t)to_len + (size_t)msg_len
        + (size_t)kem_ct_len + (size_t)nonce_len + (size_t)ciphertext_len
        + (size_t)tag_len + (size_t)sig_len;
    if (off + needed != len) {
        return -1;
    }

    if (dup_c_string(buf + off, sender_len, &out->sender_id) != 0) {
        return -1;
    }
    off += sender_len;
    if (dup_c_string(buf + off, to_len, &out->to_id) != 0) {
        fuzz_case_free(out);
        return -1;
    }
    off += to_len;
    if (dup_c_string(buf + off, msg_len, &out->msg_type) != 0) {
        fuzz_case_free(out);
        return -1;
    }
    off += msg_len;

    out->kem_ct = buf + off;
    out->kem_ct_len = kem_ct_len;
    off += kem_ct_len;
    out->nonce = buf + off;
    out->nonce_len = nonce_len;
    off += nonce_len;
    out->ciphertext = buf + off;
    out->ciphertext_len = ciphertext_len;
    off += ciphertext_len;
    out->tag = buf + off;
    out->tag_len = tag_len;
    off += tag_len;
    out->sig = buf + off;
    out->sig_len = sig_len;
    return 0;
}

static int run_case(const fixture_t *fixture, const uint8_t *data, size_t len, int expect_success) {
    fuzz_case_t fc;
    uint8_t *plaintext = NULL;
    size_t plaintext_len = 0;
    int rc;

    if (!fixture || !data) {
        return expect_success ? 1 : 0;
    }
    if (parse_input(data, len, &fc) != 0) {
        return expect_success ? 1 : 0;
    }

    rc = nxms_ms_verify_decrypt(
        fc.sender_id,
        fc.to_id,
        fc.msg_type,
        fc.escrow_id,
        fc.seq,
        fc.kem_ct,
        fc.kem_ct_len,
        fc.nonce,
        fc.nonce_len,
        fc.ciphertext,
        fc.ciphertext_len,
        fc.tag,
        fc.tag_len,
        fc.sig,
        fc.sig_len,
        fixture->recipient_sk_kem,
        fixture->recipient_sk_kem_len,
        fixture->sender_pk_sig,
        fixture->sender_pk_sig_len,
        &plaintext,
        &plaintext_len
    );

    if (plaintext) {
        nxms_ms_free_secure(plaintext, plaintext_len);
    }
    fuzz_case_free(&fc);
    if (expect_success) {
        return rc == 0 ? 0 : 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *data = NULL;
    size_t len = 0;
    int rc;
    int expect_success = 0;
    const char *fixture_path;
    const char *input_path;

    if (argc == 4 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        fixture_path = argv[2];
        input_path = argv[3];
    } else if (argc == 3) {
        fixture_path = argv[1];
        input_path = argv[2];
    } else {
        fprintf(stderr, "usage: %s <fixture.bin> <input.bin>\\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <fixture.bin> <input.bin>\\n", argv[0]);
        return 2;
    }

    if (load_fixture(fixture_path, &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\\n", fixture_path);
        return 2;
    }
    data = read_file(input_path, &len);
    if (!data) {
        fixture_free(&fixture);
        fprintf(stderr, "failed to read input: %s\\n", input_path);
        return 2;
    }

    rc = run_case(&fixture, data, len, expect_success);
    if (expect_success && rc == 0) {
        fprintf(stdout, "self-test ok\\n");
    }

    free(data);
    fixture_free(&fixture);
    return rc;
}
