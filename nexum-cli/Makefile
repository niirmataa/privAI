CC ?= cc

PKG_DEPS := libsodium jansson libcurl sqlite3 liboqs
PKG_HINT := libsodium-dev jansson-dev curl-dev sqlite-dev liboqs-dev

CFLAGS += -std=c11 -D_GNU_SOURCE -Wall -Wextra -O2
CFLAGS += -DFF_FALCON_LOGN=10 -DFALCON_FPEMU=1 -DFALCON_FPNATIVE=0
CFLAGS += -Isrc -I../nxms-transport/native/vendor/falcon

CLI_SRCS := \
	src/auth.c \
	src/cli_app.c \
	src/cli_auth_core_cmds.c \
	src/cli_common.c \
	src/cli_config.c \
	src/cli_ext_cmds.c \
	src/dm.c \
	src/http.c \
	src/json_min.c \
	src/main.c \
	src/pow.c \
	src/pqc_falcon.c \
	src/pqc_kem.c \
	src/prekeys.c \
	src/util.c \
	src/vault.c

FALCON_SRCS := \
	../nxms-transport/native/vendor/falcon/codec.c \
	../nxms-transport/native/vendor/falcon/common.c \
	../nxms-transport/native/vendor/falcon/falcon.c \
	../nxms-transport/native/vendor/falcon/fft.c \
	../nxms-transport/native/vendor/falcon/fpr.c \
	../nxms-transport/native/vendor/falcon/keygen.c \
	../nxms-transport/native/vendor/falcon/rng.c \
	../nxms-transport/native/vendor/falcon/shake.c \
	../nxms-transport/native/vendor/falcon/sign.c \
	../nxms-transport/native/vendor/falcon/vrfy.c

SRCS := $(CLI_SRCS) $(FALCON_SRCS)
OBJS := $(SRCS:.c=.o)
BIN := nexum

.PHONY: all clean deps-check print-config

all: $(BIN)

deps-check:
	@pkg-config --exists $(PKG_DEPS) || { \
		echo "Missing Alpine dev packages for nexum-cli."; \
		echo "Install: $(PKG_HINT)"; \
		exit 1; \
	}

print-config:
	@echo "CC=$(CC)"
	@echo "PKG_DEPS=$(PKG_DEPS)"
	@pkg-config --cflags $(PKG_DEPS)
	@pkg-config --libs $(PKG_DEPS)

$(BIN): deps-check $(OBJS)
	$(CC) $(CFLAGS) -o $@ $(OBJS) $(shell pkg-config --libs $(PKG_DEPS)) -lm

%.o: %.c
	$(CC) $(CFLAGS) $(shell pkg-config --cflags $(PKG_DEPS) 2>/dev/null) -c $< -o $@

clean:
	rm -f $(OBJS) $(BIN)
