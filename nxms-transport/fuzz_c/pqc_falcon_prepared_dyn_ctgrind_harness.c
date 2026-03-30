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
    uint8_t sig[FALCON_SIG_CT_SIZE(FF_FALCON_LOGN)];
    ff_falcon_prepared_sk_t prepared;
    size_t sk_len = sizeof sk;
    size_t pk_len = sizeof pk;
    size_t sig_len = sizeof sig;

    fill_seed(keygen_seed, 0x33);
    fill_seed(sign_seed, 0x44);
    fill_msg(msg);

    if (ff_falcon_keygen_seeded(keygen_seed, sizeof keygen_seed,
                                sk, &sk_len, pk, &pk_len) != 0) {
        fprintf(stderr, "ff_falcon_keygen_seeded failed\n");
        return 1;
    }
    if (ff_falcon_prepare_sk(sk, sk_len, &prepared) != 0) {
        fprintf(stderr, "ff_falcon_prepare_sk failed\n");
        return 1;
    }

    if (argc > 1 && strcmp(argv[1], "sk") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(((uint8_t *)&prepared) + 8,
                                    sizeof prepared - 8);
    } else if (argc > 1 && strcmp(argv[1], "msg") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(msg, sizeof msg);
    } else if (argc > 1 && strcmp(argv[1], "sk-msg") == 0) {
        VALGRIND_MAKE_MEM_UNDEFINED(((uint8_t *)&prepared) + 8,
                                    sizeof prepared - 8);
        VALGRIND_MAKE_MEM_UNDEFINED(msg, sizeof msg);
    }

    if (ff_falcon_sign_ct_prepared_seeded(sign_seed, sizeof sign_seed,
                                          &prepared,
                                          msg, sizeof msg,
                                          sig, &sig_len) != 0) {
        fprintf(stderr, "ff_falcon_sign_ct_prepared_seeded failed\n");
        return 1;
    }

    VALGRIND_MAKE_MEM_DEFINED(sig, sig_len);
    VALGRIND_MAKE_MEM_DEFINED(msg, sizeof msg);
    VALGRIND_MAKE_MEM_DEFINED(&prepared, sizeof prepared);
    VALGRIND_MAKE_MEM_DEFINED(pk, pk_len);
    ff_falcon_clear_prepared_sk(&prepared);
    return 0;
}
