# nexum-cli

Legacy user CLI for local vault, auth, prekeys and DM flows.

Current direction:
- user-side CLI stays useful as a local/operator-adjacent recovery tool
- escrow/operator-heavy flows are legacy and should not drive new architecture
- Falcon signing uses the same CT backend policy as `nxms-transport`

## Alpine build

This tree shares Falcon reference sources from:
- `../nxms-transport/native/vendor/falcon`

Build on Alpine:

```sh
apk add build-base libsodium-dev jansson-dev curl-dev sqlite-dev liboqs-dev
cd /home/nxms-server/privAI/nexum-cli
make
```

Helpful targets:

```sh
make deps-check
make print-config
make clean
```

## Security note

The CLI now prefers a prepared Falcon signer context for hot signing paths:
- vault load prepares signer state once
- request-time signing uses prepared `sign_dyn`
- raw encoded secret-key signing remains only as a fallback path

That keeps the runtime path aligned with the audited CT direction without changing wire semantics.
