#include "pqc_kem.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FIXTURE_MAGIC "NXMSFIX1"
#define INPUT_MAGIC   "NXMSINP1"
#define NXMS_ESCROW_ID_LEN 16
#define FRODOKEM_640_SHAKE "FrodoKEM-640-SHAKE"

typedef struct {
    uint8_t *recipient_sk_kem;
    size_t recipient_sk_kem_len;
} fixture_t;

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

static void fixture_free(fixture_t *fixture) {
    if (!fixture) {
        return;
    }
    secure_free(fixture->recipient_sk_kem, fixture->recipient_sk_kem_len);
    fixture->recipient_sk_kem = NULL;
    fixture->recipient_sk_kem_len = 0;
}

static int load_fixture(const char *path, fixture_t *out) {
    uint8_t *buf = NULL;
    size_t len = 0;
    size_t off = 0;
    uint32_t sk_len;

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
    free(buf);
    return 0;
}

static int extract_kem_ct_from_nxms_input(const uint8_t *buf, size_t len, const uint8_t **ct, size_t *ct_len) {
    size_t off = 0;
    uint16_t sender_len, to_len, msg_len;
    uint16_t kem_ct_len, nonce_len, ciphertext_len, tag_len, sig_len;
    size_t needed;

    if (!buf || !ct || !ct_len) {
        return -1;
    }
    *ct = NULL;
    *ct_len = 0;

    if (len < 8 + 8 + NXMS_ESCROW_ID_LEN + 16) {
        return -1;
    }
    if (memcmp(buf, INPUT_MAGIC, 8) != 0) {
        return -1;
    }

    off = 8;                  /* magic */
    off += 8;                 /* seq */
    off += NXMS_ESCROW_ID_LEN;

    sender_len = read_u16_le(buf + off); off += 2;
    to_len = read_u16_le(buf + off); off += 2;
    msg_len = read_u16_le(buf + off); off += 2;
    kem_ct_len = read_u16_le(buf + off); off += 2;
    nonce_len = read_u16_le(buf + off); off += 2;
    ciphertext_len = read_u16_le(buf + off); off += 2;
    tag_len = read_u16_le(buf + off); off += 2;
    sig_len = read_u16_le(buf + off); off += 2;

    needed = off
        + (size_t)sender_len
        + (size_t)to_len
        + (size_t)msg_len
        + (size_t)kem_ct_len
        + (size_t)nonce_len
        + (size_t)ciphertext_len
        + (size_t)tag_len
        + (size_t)sig_len;
    if (needed > len) {
        return -1;
    }

    off += (size_t)sender_len + (size_t)to_len + (size_t)msg_len;
    *ct = buf + off;
    *ct_len = (size_t)kem_ct_len;
    return 0;
}

static int run_case(const fixture_t *fixture, const uint8_t *buf, size_t len) {
    const uint8_t *kem_ct = NULL;
    size_t kem_ct_len = 0;
    uint8_t *ss = NULL;
    size_t ss_len = 0;
    int rc;

    if (!fixture || !buf) {
        return -1;
    }

    if (extract_kem_ct_from_nxms_input(buf, len, &kem_ct, &kem_ct_len) != 0) {
        kem_ct = buf;
        kem_ct_len = len;
    }

    rc = ff_kem_decaps(
        FRODOKEM_640_SHAKE,
        fixture->recipient_sk_kem,
        fixture->recipient_sk_kem_len,
        kem_ct,
        kem_ct_len,
        &ss,
        &ss_len
    );

    if (ss) {
        secure_free(ss, ss_len);
    }
    return rc;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *input_buf = NULL;
    size_t input_len = 0;
    int rc = 0;

    memset(&fixture, 0, sizeof(fixture));

    if (argc == 4 && strcmp(argv[1], "--self-test") == 0) {
        if (load_fixture(argv[2], &fixture) != 0) {
            fprintf(stderr, "failed to load fixture: %s\n", argv[2]);
            return 2;
        }
        input_buf = read_file(argv[3], &input_len);
        if (!input_buf) {
            fprintf(stderr, "failed to read input: %s\n", argv[3]);
            fixture_free(&fixture);
            return 2;
        }
        run_case(&fixture, input_buf, input_len);
        secure_free(input_buf, input_len);
        fixture_free(&fixture);
        puts("self-test ok");
        return 0;
    }

    if (argc != 3) {
        fprintf(stderr, "usage: %s <fixture.bin> <input.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <fixture.bin> <input.bin>\n", argv[0]);
        return 2;
    }

    if (load_fixture(argv[1], &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", argv[1]);
        return 2;
    }
    input_buf = read_file(argv[2], &input_len);
    if (!input_buf) {
        fprintf(stderr, "failed to read input: %s\n", argv[2]);
        fixture_free(&fixture);
        return 2;
    }

    run_case(&fixture, input_buf, input_len);
    secure_free(input_buf, input_len);
    fixture_free(&fixture);
    return 0;
}
