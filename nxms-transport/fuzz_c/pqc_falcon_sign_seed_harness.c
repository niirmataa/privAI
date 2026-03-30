#include "pqc_falcon.h"
#include "falcon.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FIXTURE_MAGIC "NXMSFAL1"

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

static int sign_and_verify(
    const fixture_t *fixture,
    size_t variant_index,
    const uint8_t seed[48],
    const uint8_t *msg,
    size_t msg_len
) {
    shake256_context rng;
    uint8_t sig[FF_FALCON_SIG_MAX];
    size_t sig_len;
    void *tmp_sign = NULL;
    void *tmp_verify = NULL;
    size_t tmp_sign_len;
    size_t tmp_verify_len;
    int rc;

    if (!fixture || !fixture->sender_sk_sig || !fixture->sender_pk_sig || !msg) {
        return 1;
    }
    if (variant_index >= fixture->message_count) {
        return 1;
    }

    shake256_init_prng_from_seed(&rng, seed, 48);
    sig_len = sizeof(sig);

    tmp_sign_len = FALCON_TMPSIZE_SIGNDYN(FF_FALCON_LOGN);
    tmp_sign = malloc(tmp_sign_len);
    if (!tmp_sign) {
        return 1;
    }
    memset(tmp_sign, 0, tmp_sign_len);

    rc = falcon_sign_dyn(
        &rng,
        sig,
        &sig_len,
        FALCON_SIG_CT,
        fixture->sender_sk_sig,
        fixture->sender_sk_sig_len,
        msg,
        msg_len,
        tmp_sign,
        tmp_sign_len
    );
    free(tmp_sign);
    if (rc != 0) {
        return 1;
    }

    tmp_verify_len = FALCON_TMPSIZE_VERIFY(FF_FALCON_LOGN);
    tmp_verify = malloc(tmp_verify_len);
    if (!tmp_verify) {
        return 1;
    }
    memset(tmp_verify, 0, tmp_verify_len);

    rc = falcon_verify(
        sig,
        sig_len,
        FALCON_SIG_CT,
        fixture->sender_pk_sig,
        fixture->sender_pk_sig_len,
        msg,
        msg_len,
        tmp_verify,
        tmp_verify_len
    );
    free(tmp_verify);
    return rc == 0 ? 0 : 1;
}

static int run_case(const fixture_t *fixture, const uint8_t *input, size_t input_len, int expect_success) {
    size_t variant_index = 0;
    uint8_t seed[48];
    const uint8_t *msg = NULL;
    size_t msg_len = 0;

    if (!fixture || fixture->message_count == 0) {
        return 1;
    }

    memset(seed, 0, sizeof seed);

    if (input_len > 0) {
        variant_index = (size_t)input[0] % fixture->message_count;
    }

    if (input_len > 1) {
        size_t seed_copy = input_len - 1;
        if (seed_copy > sizeof seed) {
            seed_copy = sizeof seed;
        }
        memcpy(seed, input + 1, seed_copy);
    }

    if (input_len > 1 + sizeof seed) {
        msg = input + 1 + sizeof seed;
        msg_len = input_len - 1 - sizeof seed;
    } else {
        msg = fixture->messages[variant_index];
        msg_len = fixture->message_lens[variant_index];
    }

    if (expect_success) {
        memset(seed, 0x42, sizeof seed);
        msg = fixture->messages[variant_index];
        msg_len = fixture->message_lens[variant_index];
    }

    return sign_and_verify(fixture, variant_index, seed, msg, msg_len);
}

int main(int argc, char **argv) {
    fixture_t fixture;
    uint8_t *input = NULL;
    size_t input_len = 0;
    int expect_success = 0;
    int rc;
    const char *fixture_path;
    const char *input_path;

    memset(&fixture, 0, sizeof(fixture));

    if (argc == 4 && strcmp(argv[1], "--self-test") == 0) {
        expect_success = 1;
        fixture_path = argv[2];
        input_path = argv[3];
    } else if (argc == 3) {
        fixture_path = argv[1];
        input_path = argv[2];
    } else {
        fprintf(stderr, "usage: %s <falcon_fixture.bin> <input.bin>\n", argv[0]);
        fprintf(stderr, "   or: %s --self-test <falcon_fixture.bin> <input.bin>\n", argv[0]);
        return 2;
    }

    if (load_fixture(fixture_path, &fixture) != 0) {
        fprintf(stderr, "failed to load fixture: %s\n", fixture_path);
        return 2;
    }

    if (strcmp(input_path, "/dev/null") != 0) {
        input = read_file(input_path, &input_len);
        if (!input) {
            fixture_free(&fixture);
            fprintf(stderr, "failed to read input: %s\n", input_path);
            return 2;
        }
    }

    rc = run_case(&fixture, input, input_len, expect_success);
    secure_free(input, input_len);
    fixture_free(&fixture);
    if (expect_success && rc == 0) {
        fprintf(stderr, "self-test ok\n");
    }
    return rc;
}
