#include "nxms_ms_transport.h"

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
    size_t seq_off;
    size_t seq_len;
    size_t escrow_off;
    size_t escrow_len;
    size_t sender_off;
    size_t sender_len;
    size_t to_off;
    size_t to_len;
    size_t msg_off;
    size_t msg_len;
    size_t kem_off;
    size_t kem_len;
    size_t nonce_off;
    size_t nonce_len;
    size_t ciphertext_off;
    size_t ciphertext_len;
    size_t tag_off;
    size_t tag_len;
    size_t sig_off;
    size_t sig_len;
} packet_layout_t;

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

enum {
    TARGET_SEQ = 1,
    TARGET_ESCROW = 2,
    TARGET_SENDER = 3,
    TARGET_TO = 4,
    TARGET_MSG = 5,
    TARGET_KEM = 6,
    TARGET_NONCE = 7,
    TARGET_CIPHERTEXT = 8,
    TARGET_TAG = 9,
    TARGET_SIG = 10
};

enum {
    OP_XOR = 0,
    OP_SET = 1,
    OP_ADD = 2,
    OP_ZERO = 3
};

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

static int parse_layout(const uint8_t *buf, size_t len, packet_layout_t *layout) {
    size_t off = 0;
    uint16_t sender_len, to_len, msg_len;
    uint16_t kem_ct_len, nonce_len, ciphertext_len, tag_len, sig_len;
    size_t needed;

    if (!buf || !layout || len < 8 + 8 + NXMS_ESCROW_ID_LEN + 16) {
        return -1;
    }
    memset(layout, 0, sizeof(*layout));
    if (memcmp(buf, INPUT_MAGIC, 8) != 0) {
        return -1;
    }

    layout->seq_off = 8;
    layout->seq_len = 8;
    off = layout->seq_off + layout->seq_len;
    layout->escrow_off = off;
    layout->escrow_len = NXMS_ESCROW_ID_LEN;
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
    if (needed != len) {
        return -1;
    }

    layout->sender_off = off;
    layout->sender_len = sender_len;
    off += sender_len;
    layout->to_off = off;
    layout->to_len = to_len;
    off += to_len;
    layout->msg_off = off;
    layout->msg_len = msg_len;
    off += msg_len;
    layout->kem_off = off;
    layout->kem_len = kem_ct_len;
    off += kem_ct_len;
    layout->nonce_off = off;
    layout->nonce_len = nonce_len;
    off += nonce_len;
    layout->ciphertext_off = off;
    layout->ciphertext_len = ciphertext_len;
    off += ciphertext_len;
    layout->tag_off = off;
    layout->tag_len = tag_len;
    off += tag_len;
    layout->sig_off = off;
    layout->sig_len = sig_len;
    return 0;
}

static int select_target(packet_layout_t *layout, uint8_t **field, size_t *field_len,
                         uint8_t *packet, uint8_t target) {
    if (!layout || !field || !field_len || !packet) {
        return -1;
    }
    *field = NULL;
    *field_len = 0;

    switch (target) {
        case TARGET_SEQ:
            *field = packet + layout->seq_off;
            *field_len = layout->seq_len;
            return 0;
        case TARGET_ESCROW:
            *field = packet + layout->escrow_off;
            *field_len = layout->escrow_len;
            return 0;
        case TARGET_SENDER:
            *field = packet + layout->sender_off;
            *field_len = layout->sender_len;
            return 0;
        case TARGET_TO:
            *field = packet + layout->to_off;
            *field_len = layout->to_len;
            return 0;
        case TARGET_MSG:
            *field = packet + layout->msg_off;
            *field_len = layout->msg_len;
            return 0;
        case TARGET_KEM:
            *field = packet + layout->kem_off;
            *field_len = layout->kem_len;
            return 0;
        case TARGET_NONCE:
            *field = packet + layout->nonce_off;
            *field_len = layout->nonce_len;
            return 0;
        case TARGET_CIPHERTEXT:
            *field = packet + layout->ciphertext_off;
            *field_len = layout->ciphertext_len;
            return 0;
        case TARGET_TAG:
            *field = packet + layout->tag_off;
            *field_len = layout->tag_len;
            return 0;
        case TARGET_SIG:
            *field = packet + layout->sig_off;
            *field_len = layout->sig_len;
            return 0;
        default:
            return -1;
    }
}

static void apply_program(uint8_t *packet, const packet_layout_t *layout,
                          const uint8_t *prog, size_t prog_len) {
    size_t ops;
    size_t i;

    if (!packet || !layout || !prog) {
        return;
    }

    ops = prog_len / 5u;
    if (ops > 128u) {
        ops = 128u;
    }

    for (i = 0; i < ops; i++) {
        const uint8_t *rec = prog + (i * 5u);
        uint8_t *field = NULL;
        size_t field_len = 0;
        size_t offset = (size_t)rec[2] | ((size_t)rec[3] << 8);
        uint8_t value = rec[4];

        if (select_target((packet_layout_t *)layout, &field, &field_len, packet, rec[0]) != 0) {
            continue;
        }
        if (field_len == 0 || offset >= field_len) {
            continue;
        }

        switch (rec[1] & 0x03u) {
            case OP_XOR:
                field[offset] ^= value;
                break;
            case OP_SET:
                field[offset] = value;
                break;
            case OP_ADD:
                field[offset] = (uint8_t)(field[offset] + value);
                break;
            case OP_ZERO:
                field[offset] = 0;
                break;
        }
    }
}

static int run_case(const fixture_t *fixture,
                    const uint8_t *base_packet, size_t base_packet_len,
                    const packet_layout_t *layout,
                    const uint8_t *program, size_t program_len,
                    int expect_success) {
    fuzz_case_t fc;
    uint8_t *plaintext = NULL;
    size_t plaintext_len = 0;
    uint8_t *packet_copy = NULL;
    int rc = 0;

    if (!fixture || !base_packet || !layout) {
        return expect_success ? 1 : 0;
    }

    packet_copy = (uint8_t *)malloc(base_packet_len ? base_packet_len : 1u);
    if (!packet_copy) {
        return expect_success ? 1 : 0;
    }
    memcpy(packet_copy, base_packet, base_packet_len);
    if (program && program_len) {
        apply_program(packet_copy, layout, program, program_len);
    }

    if (parse_input(packet_copy, base_packet_len, &fc) == 0) {
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
    } else {
        rc = -1;
    }

    free(packet_copy);
    if (expect_success) {
        return rc == 0 ? 0 : 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *base_packet = NULL;
    size_t base_packet_len = 0;
    packet_layout_t layout;
    uint8_t *program = NULL;
    size_t program_len = 0;
    int expect_success = 0;
    int rc;
    const char *fixture_path;
    const char *base_packet_path;
    const char *program_path;

    memset(&fixture, 0, sizeof(fixture));
    memset(&layout, 0, sizeof(layout));

    if (argc == 5 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        fixture_path = argv[2];
        base_packet_path = argv[3];
        program_path = argv[4];
    } else if (argc == 4) {
        fixture_path = argv[1];
        base_packet_path = argv[2];
        program_path = argv[3];
    } else {
        fprintf(stderr, "usage: %s <fixture.bin> <base_packet.bin> <program.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <fixture.bin> <base_packet.bin> <program.bin>\n", argv[0]);
        return 2;
    }

    if (load_fixture(fixture_path, &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", fixture_path);
        return 2;
    }

    base_packet = read_file(base_packet_path, &base_packet_len);
    if (!base_packet || parse_layout(base_packet, base_packet_len, &layout) != 0) {
        fixture_free(&fixture);
        free(base_packet);
        fprintf(stderr, "failed to load base packet: %s\n", base_packet_path);
        return 2;
    }

    program = read_file(program_path, &program_len);
    if (!program && strcmp(program_path, "/dev/null") != 0) {
        fixture_free(&fixture);
        free(base_packet);
        fprintf(stderr, "failed to read program: %s\n", program_path);
        return 2;
    }

    rc = run_case(&fixture, base_packet, base_packet_len, &layout, program, program_len, expect_success);
    if (expect_success && rc == 0) {
        puts("self-test ok");
    }

    secure_free(program, program_len);
    free(base_packet);
    fixture_free(&fixture);
    return rc;
}
