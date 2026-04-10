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

## Escrow helpers

For operator and emergency workflows, a unified `escrow-confirm` wrapper is available:
```sh
# Convenience wrapper for legacy release confirmation
nexum escrow-confirm --action release --base <url> --id <escrow_id> --nick <nick> --token <tok> --txid <64hex>

# Convenience wrapper for legacy refund confirmation
nexum escrow-confirm --action refund --base <url> --id <escrow_id> --nick <nick> --token <tok> --txid <64hex>
```
Legacy commands `escrow-confirm-release` and `escrow-confirm-refund` are fully preserved.

For settlement submission flows, a unified `escrow-settle` wrapper is available:
```sh
# Wrapper for release settlement, including signer-specific arguments
nexum escrow-settle --action release --base <url> --id <escrow_id> --nick <nick> --token <tok> --tx-data-hex <hex> --signer-wallet-password-env PW

# Wrapper for refund settlement, restricted to refund-applicable arguments
nexum escrow-settle --action refund --base <url> --id <escrow_id> --nick <nick> --token <tok> --tx-data-hex <hex> --signer-action-token-env TOK
```
Legacy commands `escrow-release` and `escrow-refund` are fully preserved.
