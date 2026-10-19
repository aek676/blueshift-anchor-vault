# Blueshift Anchor Vault

A minimal SOL vault written in [Anchor](https://www.anchor-lang.com/), solving the
[Blueshift](https://learn.blueshift.gg/) *Anchor Vault* challenge. Each user gets their own
vault, a PDA owned by the System Program, where they can lock lamports and later withdraw them.

## Program

Both instructions share the `VaultAction` accounts:

| Account | Description |
|---|---|
| `signer` | Vault owner; pays the deposit and receives the withdrawal (mutable signer). |
| `vault` | PDA derived from `["vault", signer]`; a plain `SystemAccount` holding the lamports. |
| `system_program` | Used for the transfers via CPI. |

### `deposit(amount: u64)`

Transfers `amount` lamports from the signer to their vault.

- Fails with `VaultAlreadyExists` if the vault already holds lamports.
- Fails with `InvalidAmount` unless `amount` is greater than the rent-exempt minimum for a
  0-byte account, so the vault is never left below rent exemption.

### `withdraw()`

Transfers the vault's entire balance back to the signer, signing the CPI with the vault's
PDA seeds. Fails with `InvalidAmount` if the vault is empty. Once emptied, the vault can be
reused with a new deposit.

## Build and test

Requires the Anchor CLI 1.2.1 and Solana CLI 4.1.2 (pinned in `Anchor.toml`) and Rust 1.89
(pinned in `rust-toolchain.toml`).

```bash
anchor build
anchor test   # runs `cargo test --locked`
```

The tests in `programs/blueshift_anchor_vault/tests/` use [LiteSVM](https://github.com/LiteSVM/litesvm)
to load the compiled program in-process, covering the happy paths, the error cases above,
and isolation between vaults of different signers.

### Dependencies

`litesvm = "=0.15.0"` only builds against the Agave crate versions pinned in `Cargo.lock`.
**Don't run a blanket `cargo update`**: update specific crates with `cargo update -p <crate>`.
