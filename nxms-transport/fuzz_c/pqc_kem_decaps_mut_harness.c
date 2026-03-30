#include "pqc_kem.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FIXTURE_MAGIC "NXMSFIX1"
#define FRODOKEM_640_SHAKE "FrodoKEM-640-SHAKE"

enum {
    OP_XOR = 0,
    OP_SET = 1,
    OP_ADD = 2,
    OP_ZERO = 3
};

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

static void apply_program(uint8_t *kem_ct, size_t kem_ct_len, const uint8_t *program, size_t program_len) {
    size_t ops;
    size_t i;

    if (!kem_ct || !program) {
        return;
    }

    ops = program_len / 4u;
    if (ops > 128u) {
        ops = 128u;
    }

    for (i = 0; i < ops; i++) {
        const uint8_t *rec = program + (i * 4u);
        size_t offset = (size_t)rec[1] | ((size_t)rec[2] << 8);
        uint8_t value = rec[3];

        if (kem_ct_len == 0 || offset >= kem_ct_len) {
            continue;
        }

        switch (rec[0] & 0x03u) {
            case OP_XOR:
                kem_ct[offset] ^= value;
                break;
            case OP_SET:
                kem_ct[offset] = value;
                break;
            case OP_ADD:
                kem_ct[offset] = (uint8_t)(kem_ct[offset] + value);
                break;
            case OP_ZERO:
                kem_ct[offset] = 0;
                break;
        }
    }
}

static int run_case(const fixture_t *fixture,
                    const uint8_t *base_ct, size_t base_ct_len,
                    const uint8_t *program, size_t program_len,
                    int expect_success) {
    uint8_t *kem_ct = NULL;
    uint8_t *ss = NULL;
    size_t ss_len = 0;
    int rc;

    if (!fixture || !base_ct) {
        return expect_success ? 1 : 0;
    }

    kem_ct = (uint8_t *)malloc(base_ct_len ? base_ct_len : 1u);
    if (!kem_ct) {
        return expect_success ? 1 : 0;
    }
    memcpy(kem_ct, base_ct, base_ct_len);
    if (program && program_len) {
        apply_program(kem_ct, base_ct_len, program, program_len);
    }

    rc = ff_kem_decaps(
        FRODOKEM_640_SHAKE,
        fixture->recipient_sk_kem,
        fixture->recipient_sk_kem_len,
        kem_ct,
        base_ct_len,
        &ss,
        &ss_len
    );

    secure_free(kem_ct, base_ct_len);
    if (ss) {
        secure_free(ss, ss_len);
    }

    if (expect_success) {
        return rc == 0 ? 0 : 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *base_ct = NULL;
    size_t base_ct_len = 0;
    uint8_t *program = NULL;
    size_t program_len = 0;
    int expect_success = 0;
    int rc;
    const char *fixture_path;
    const char *base_ct_path;
    const char *program_path;

    memset(&fixture, 0, sizeof(fixture));

    if (argc == 5 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        fixture_path = argv[2];
        base_ct_path = argv[3];
        program_path = argv[4];
    } else if (argc == 4) {
        fixture_path = argv[1];
        base_ct_path = argv[2];
        program_path = argv[3];
    } else {
        fprintf(stderr, "usage: %s <fixture.bin> <base_kem_ct.bin> <program.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <fixture.bin> <base_kem_ct.bin> <program.bin>\n", argv[0]);
        return 2;
    }

    if (load_fixture(fixture_path, &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", fixture_path);
        return 2;
    }

    base_ct = read_file(base_ct_path, &base_ct_len);
    if (!base_ct) {
        fixture_free(&fixture);
        fprintf(stderr, "failed to read base kem_ct: %s\n", base_ct_path);
        return 2;
    }

    program = read_file(program_path, &program_len);
    if (!program && strcmp(program_path, "/dev/null") != 0) {
        secure_free(base_ct, base_ct_len);
        fixture_free(&fixture);
        fprintf(stderr, "failed to read program: %s\n", program_path);
        return 2;
    }

    rc = run_case(&fixture, base_ct, base_ct_len, program, program_len, expect_success);
    if (expect_success && rc == 0) {
        puts("self-test ok");
    }

    secure_free(program, program_len);
    secure_free(base_ct, base_ct_len);
    fixture_free(&fixture);
    return rc;
}
