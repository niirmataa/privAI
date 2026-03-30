#include "nxms_ms_transport.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define INPUT_MAGIC "NXMSINP1"

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

static uint16_t read_u16_le(const uint8_t *p) {
    return (uint16_t)p[0] | ((uint16_t)p[1] << 8);
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

static int run_case(const uint8_t *data, size_t len, int expect_success) {
    fuzz_case_t fc;
    int rc;

    if (!data) {
        return expect_success ? 1 : 0;
    }
    rc = parse_input(data, len, &fc);
    if (rc == 0) {
        fuzz_case_free(&fc);
    }
    if (expect_success) {
        return rc == 0 ? 0 : 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    uint8_t *data = NULL;
    size_t len = 0;
    int expect_success = 0;
    int rc;
    const char *input_path;

    if (argc == 3 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        input_path = argv[2];
    } else if (argc == 2) {
        input_path = argv[1];
    } else {
        fprintf(stderr, "usage: %s <input.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <input.bin>\n", argv[0]);
        return 2;
    }

    data = read_file(input_path, &len);
    if (!data) {
        fprintf(stderr, "failed to read input: %s\n", input_path);
        return 2;
    }

    rc = run_case(data, len, expect_success);
    if (expect_success && rc == 0) {
        puts("self-test ok");
    }

    free(data);
    return rc;
}
