#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <valgrind/memcheck.h>

#include "../native/nexum_cli_src/pqc_falcon.h"
#include "../native/vendor/falcon/falcon.h"

enum {
    SEED_LEN = 48,
    MSG_LEN = 64
};

static void
fill_seed(uint8_t *dst, uint8_t value)
{
    memset(dst, value, SEED_LEN);
}

static void
fill_msg(uint8_t *dst)
{
    for (size_t i = 0; i < MSG_LEN; i++) {
        dst[i] = (uint8_t)(0x5Au ^ (uint8_t)(i * 19u));
    }
}

int
main(int argc, char **argv)
{
    uint8_t keygen_seed[SEED_LEN];
    uint8_t sign_seed[SEED_LEN];
    uint8_t msg[MSG_LEN];
    uint8_t sk[FALCON_PRIVKEY_SIZE(FF_FALCON_LOGN)];
    uint8_t pk[FALCON_PUBKEY_SIZE(FF_FALCON_LOGN)];
    size_t sk_len = sizeof sk;
    size_t pk_len = sizeof pk;
    uint8_t sig[FALCON_SIG_CT_SIZE(FF_FALCON_LOGN)];
    size_t sig_len = sizeof sig;
    uint8_t *expanded = NULL;
    void *expand_tmp = NULL;
    void *sign_tmp = NULL;
    size_t expanded_len = FALCON_EXPANDEDKEY_SIZE(FF_FALCON_LOGN);
    size_t expand_tmp_len = FALCON_TMPSIZE_EXPANDPRIV(FF_FALCON_LOGN);
    size_t sign_tmp_len = FALCON_TMPSIZE_SIGNTREE(FF_FALCON_LOGN);
    int rc;

    fill_seed(keygen_seed, 0x11);
    fill_seed(sign_seed, 0x22);
    fill_msg(msg);

    if (ff_falcon_keygen_seeded(keygen_seed, sizeof keygen_seed,
                                sk, &sk_len, pk, &pk_len) != 0) {
        fprintf(stderr, "ff_falcon_keygen_seeded failed\n");
        return 1;
    }

    expanded = (uint8_t *)malloc(expanded_len);
    expand_tmp = malloc(expand_tmp_len);
    sign_tmp = malloc(sign_tmp_len);
    if (!expanded || !expand_tmp || !sign_tmp) {
        fprintf(stderr, "allocation failed\n");
        free(expanded);
        free(expand_tmp);
        free(sign_tmp);
        return 1;
    }

    rc = falcon_expand_privkey(expanded, expanded_len,
                               sk, sk_len,
                               expand_tmp, expand_tmp_len);
    if (rc != 0) {
        fprintf(stderr, "falcon_expand_privkey failed\n");
        free(expanded);
        free(expand_tmp);
        free(sign_tmp);
        return 1;
    }

    if (argc > 1 && strcmp(argv[1], "expkey") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(expanded, expanded_len);
    } else if (argc > 1 && strcmp(argv[1], "msg") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(msg, sizeof msg);
    } else if (argc > 1 && strcmp(argv[1], "expkey-msg") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(expanded, expanded_len);
        VALGRIND_MAKE_MEM_UNDEFINED(msg, sizeof msg);
    }

    {
        shake256_context rng;
        shake256_init_prng_from_seed(&rng, sign_seed, sizeof sign_seed);
        shake256_flip(&rng);
        rc = falcon_sign_tree(&rng,
                              sig, &sig_len, FALCON_SIG_CT,
                              expanded,
                              msg, sizeof msg,
                              sign_tmp, sign_tmp_len);
    }
    if (rc != 0) {
        fprintf(stderr, "falcon_sign_tree failed\n");
        free(expanded);
        free(expand_tmp);
        free(sign_tmp);
        return 1;
    }

    VALGRIND_MAKE_MEM_DEFINED(sig, sig_len);
    VALGRIND_MAKE_MEM_DEFINED(msg, sizeof msg);
    VALGRIND_MAKE_MEM_DEFINED(expanded, expanded_len);
    VALGRIND_MAKE_MEM_DEFINED(pk, pk_len);

    free(expanded);
    free(expand_tmp);
    free(sign_tmp);
    return 0;
}
