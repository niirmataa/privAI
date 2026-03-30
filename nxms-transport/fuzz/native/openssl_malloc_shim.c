#include <openssl/crypto.h>

int nxms_openssl_malloc_probe(void) {
    void *p = CRYPTO_malloc(32, NULL, 0);
    if (!p) {
        return -1;
    }
    CRYPTO_free(p, NULL, 0);
    return 0;
}
