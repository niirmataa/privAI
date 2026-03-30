#include "pqc_falcon.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FIXTURE_MAGIC "NXMSFAL1"

enum {
    TARGET_MSG = 5,
    TARGET_SIG = 10,
    TARGET_PUBKEY = 11
};

enum {
    OP_XOR = 0,
    OP_SET = 1,
    OP_ADD = 2,
    OP_ZERO = 3
};

enum {
    MODE_NORMAL = 0,
    MODE_KEYGEN = 1,
    MODE_TRUNCATED_SK = 2,
    MODE_TRUNCATED_PK = 3,
    MODE_TRUNCATED_SIG = 4
};

typedef struct {
    uint8_t *sender_sk_sig;
    size_t sender_sk_sig_len;
    uint8_t *sender_pk_sig;
    size_t sender_pk_sig_len;
    uint8_t **messages;
    size_t *message_lens;
    size_t message_count;
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
    size_t i;

    if (!fixture) {
        return;
    }
    secure_free(fixture->sender_sk_sig, fixture->sender_sk_sig_len);
    fixture->sender_sk_sig = NULL;
    fixture->sender_sk_sig_len = 0;
    secure_free(fixture->sender_pk_sig, fixture->sender_pk_sig_len);
    fixture->sender_pk_sig = NULL;
    fixture->sender_pk_sig_len = 0;
    if (fixture->messages) {
        for (i = 0; i < fixture->message_count; i++) {
            secure_free(fixture->messages[i], fixture->message_lens ? fixture->message_lens[i] : 0);
        }
    }
    free(fixture->messages);
    fixture->messages = NULL;
    free(fixture->message_lens);
    fixture->message_lens = NULL;
    fixture->message_count = 0;
}

static int load_fixture(const char *path, fixture_t *out) {
    uint8_t *buf = NULL;
    size_t len = 0;
    size_t off = 0;
    uint32_t sk_len, pk_len, msg_count, i;

    if (!path || !out) {
        return -1;
    }
    memset(out, 0, sizeof(*out));
    buf = read_file(path, &len);
    if (!buf) {
        return -1;
    }
    if (len < 8 + 4 + 4 + 4) {
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
    out->sender_sk_sig = (uint8_t *)malloc(sk_len ? sk_len : 1u);
    if (!out->sender_sk_sig) {
        free(buf);
        return -1;
    }
    memcpy(out->sender_sk_sig, buf + off, sk_len);
    out->sender_sk_sig_len = sk_len;
    off += sk_len;

    pk_len = read_u32_le(buf + off);
    off += 4;
    if (off + pk_len + 4 > len) {
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
    off += pk_len;

    msg_count = read_u32_le(buf + off);
    off += 4;
    out->messages = (uint8_t **)calloc(msg_count ? msg_count : 1u, sizeof(*out->messages));
    out->message_lens = (size_t *)calloc(msg_count ? msg_count : 1u, sizeof(*out->message_lens));
    if (!out->messages || !out->message_lens) {
        fixture_free(out);
        free(buf);
        return -1;
    }
    out->message_count = msg_count;

    for (i = 0; i < msg_count; i++) {
        uint32_t msg_len;
        if (off + 4 > len) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        msg_len = read_u32_le(buf + off);
        off += 4;
        if (off + msg_len > len) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        out->messages[i] = (uint8_t *)malloc(msg_len ? msg_len : 1u);
        if (!out->messages[i]) {
            fixture_free(out);
            free(buf);
            return -1;
        }
        memcpy(out->messages[i], buf + off, msg_len);
        out->message_lens[i] = msg_len;
        off += msg_len;
    }

    free(buf);
    return 0;
}

static void apply_program(
    uint8_t *msg,
    size_t msg_len,
    uint8_t *sig,
    size_t sig_len,
    uint8_t *pk,
    size_t pk_len,
    const uint8_t *program,
    size_t program_len
) {
    size_t ops;
    size_t i;

    if (!program || program_len <= 1) {
        return;
    }

    ops = (program_len - 1u) / 5u;
    if (ops > 128u) {
        ops = 128u;
    }

    for (i = 0; i < ops; i++) {
        const uint8_t *rec = program + 1u + (i * 5u);
        uint8_t *target = NULL;
        size_t target_len = 0;
        size_t offset = (size_t)rec[2] | ((size_t)rec[3] << 8);
        uint8_t value = rec[4];

        switch (rec[0]) {
            case TARGET_MSG:
                target = msg;
                target_len = msg_len;
                break;
            case TARGET_SIG:
                target = sig;
                target_len = sig_len;
                break;
            case TARGET_PUBKEY:
                target = pk;
                target_len = pk_len;
                break;
            default:
                continue;
        }

        if (!target || target_len == 0 || offset >= target_len) {
            continue;
        }

        switch (rec[1] & 0x03u) {
            case OP_XOR:
                target[offset] ^= value;
                break;
            case OP_SET:
                target[offset] = value;
                break;
            case OP_ADD:
                target[offset] = (uint8_t)(target[offset] + value);
                break;
            case OP_ZERO:
                target[offset] = 0;
                break;
        }
    }
}

static int run_case(const fixture_t *fixture, const uint8_t *program, size_t program_len, int expect_success) {
    size_t mode;
    size_t variant_index;
    uint8_t *msg = NULL;
    uint8_t *pk = NULL;
    uint8_t *sk = NULL;
    uint8_t sig[FF_FALCON_SIG_MAX];
    size_t sig_len = sizeof(sig);
    size_t msg_len;
    size_t sk_len = 0;
    size_t pk_len = 0;
    int rc;

    if (!fixture || fixture->message_count == 0) {
        return expect_success ? 1 : 0;
    }

    mode = (program_len > 0) ? (((size_t)program[0] >> 5) & 0x07u) : MODE_NORMAL;
    variant_index = (program_len > 0) ? ((size_t)(program[0] & 0x1Fu) % fixture->message_count) : 0u;
    msg_len = fixture->message_lens[variant_index];

    msg = (uint8_t *)malloc(msg_len ? msg_len : 1u);
    pk = (uint8_t *)malloc(fixture->sender_pk_sig_len ? fixture->sender_pk_sig_len : 1u);
    if (!msg || !pk) {
        secure_free(msg, msg_len);
        secure_free(pk, fixture->sender_pk_sig_len);
        return expect_success ? 1 : 0;
    }
    memcpy(msg, fixture->messages[variant_index], msg_len);
    memcpy(pk, fixture->sender_pk_sig, fixture->sender_pk_sig_len);
    pk_len = fixture->sender_pk_sig_len;

    switch (mode) {
        case MODE_KEYGEN:
            sk = (uint8_t *)malloc(4096u);
            if (!sk) {
                secure_free(msg, msg_len);
                secure_free(pk, fixture->sender_pk_sig_len);
                return expect_success ? 1 : 0;
            }
            if (ff_falcon_keygen(sk, &sk_len, pk, &pk_len) != 0) {
                secure_free(sk, 4096u);
                secure_free(msg, msg_len);
                secure_free(pk, fixture->sender_pk_sig_len);
                return expect_success ? 1 : 0;
            }
            rc = ff_falcon_sign_ct(sk, sk_len, msg, msg_len, sig, &sig_len);
            secure_free(sk, 4096u);
            if (rc != 0) {
                secure_free(msg, msg_len);
                secure_free(pk, fixture->sender_pk_sig_len);
                return expect_success ? 1 : 0;
            }
            rc = ff_falcon_verify(pk, pk_len, msg, msg_len, sig, sig_len);
            secure_free(msg, msg_len);
            secure_free(pk, fixture->sender_pk_sig_len);
            return (expect_success && rc != 0) ? 1 : 0;

        case MODE_TRUNCATED_SK:
            rc = ff_falcon_sign_ct(
                fixture->sender_sk_sig,
                fixture->sender_sk_sig_len > 0 ? fixture->sender_sk_sig_len - 1u : 0u,
                msg,
                msg_len,
                sig,
                &sig_len
            );
            secure_free(msg, msg_len);
            secure_free(pk, fixture->sender_pk_sig_len);
            return (expect_success ? (rc == 0 ? 0 : 1) : 0);

        default:
            break;
    }

    rc = ff_falcon_sign_ct(
        fixture->sender_sk_sig,
        fixture->sender_sk_sig_len,
        msg,
        msg_len,
        sig,
        &sig_len
    );
    if (rc != 0) {
        secure_free(msg, msg_len);
        secure_free(pk, fixture->sender_pk_sig_len);
        return expect_success ? 1 : 0;
    }

    if (mode == MODE_TRUNCATED_PK) {
        rc = ff_falcon_verify(
            pk,
            pk_len > 0 ? pk_len - 1u : 0u,
            msg,
            msg_len,
            sig,
            sig_len
        );
        secure_free(msg, msg_len);
        secure_free(pk, fixture->sender_pk_sig_len);
        return (expect_success && rc != 0) ? 1 : 0;
    }

    if (mode == MODE_TRUNCATED_SIG) {
        rc = ff_falcon_verify(
            pk,
            pk_len,
            msg,
            msg_len,
            sig,
            sig_len > 0 ? sig_len - 1u : 0u
        );
        secure_free(msg, msg_len);
        secure_free(pk, fixture->sender_pk_sig_len);
        return (expect_success && rc != 0) ? 1 : 0;
    }

    if (program && program_len > 1) {
        apply_program(msg, msg_len, sig, sig_len, pk, fixture->sender_pk_sig_len, program, program_len);
    }

    rc = ff_falcon_verify(pk, pk_len, msg, msg_len, sig, sig_len);

    secure_free(msg, msg_len);
    secure_free(pk, fixture->sender_pk_sig_len);
    if (expect_success) {
        return rc == 0 ? 0 : 1;
    }
    return 0;
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *program = NULL;
    size_t program_len = 0;
    int expect_success = 0;
    int rc;
    const char *fixture_path;
    const char *program_path;

    memset(&fixture, 0, sizeof(fixture));

    if (argc == 4 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        fixture_path = argv[2];
        program_path = argv[3];
    } else if (argc == 3) {
        fixture_path = argv[1];
        program_path = argv[2];
    } else {
        fprintf(stderr, "usage: %s <falcon_fixture.bin> <program.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <falcon_fixture.bin> <program.bin>\n", argv[0]);
        return 2;
    }

    if (load_fixture(fixture_path, &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", fixture_path);
        return 2;
    }

    if (strcmp(program_path, "/dev/null") != 0) {
        program = read_file(program_path, &program_len);
        if (!program) {
            fixture_free(&fixture);
            fprintf(stderr, "failed to read program: %s\n", program_path);
            return 2;
        }
    }

    rc = run_case(&fixture, program, program_len, expect_success);
    secure_free(program, program_len);
    fixture_free(&fixture);
    return rc;
}
