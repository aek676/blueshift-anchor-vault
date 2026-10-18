# blueshift_anchor_vault

Generated from **anchor-template**: exactly what `anchor init` 1.2.1 creates (default
options: multi-file program + LiteSVM Rust tests), plus the fixes it is missing so it builds
and its tests pass out of the box.

## Usage

```bash
cargo install cargo-generate
cargo generate gh:<owner>/anchor-template --name my-program
cd my-program
anchor test           # build + cargo test --locked
```

### Program ID

Like `anchor init`, every generated project gets **its own program ID**. A post-generation
hook (`init-program-id.rhai`) runs:

```bash
solana-keygen new --outfile target/deploy/<program>-keypair.json && anchor keys sync
```

cargo-generate asks for confirmation before running it; pass `--allow-commands` to skip
the prompt (required with `--silent`). The keypair stays in `target/` (git-ignored); back
it up before deploying, it's the only way to upgrade the program later.

If the hook didn't run (declined, or `solana-keygen`/`anchor` not on `PATH`), the project
keeps a placeholder ID: run `anchor build && anchor keys sync` once. Tests pass either way,
but a program deployed with the placeholder fails at runtime with `DeclaredProgramIdMismatch`.

## What differs from `anchor init` 1.2.1

| File | `anchor init` 1.2.1 | This template | Why |
|---|---|---|---|
| `programs/*/Cargo.toml` | `litesvm = "0.10.0"`, `solana-message`/`solana-transaction` 3.x | `litesvm = "=0.15.0"`, `solana-message`/`solana-transaction` 4 | `anchor build` defaults to SBPFv3 and litesvm 0.10 can't load it: the generated test fails with `InvalidAccountData` ([otter-sec/anchor#5171](https://github.com/otter-sec/anchor/issues/5171)). Same deps as Anchor's master template. |
| `Cargo.lock` | not generated | committed | litesvm 0.15 only builds against the Agave crate versions it pins (newer ones break it). Taken from Anchor's master template, with `anchor-lang` bumped to 1.2.1. |
| `programs/*/Cargo.toml` | `anchor-lang = "1.2.1"` | `anchor-lang = "=1.2.1"` | Exact pin, matching the CLI. |
| `Anchor.toml` `[toolchain]` | empty | `anchor_version = "1.2.1"`, `solana_version = "4.1.2"` | Anchor 1.2 recommends Solana 4.1.2, but avm 1.2.1 switches your *global* Solana CLI to 3.1.10 on every `anchor` call unless pinned ([otter-sec/anchor#5031](https://github.com/otter-sec/anchor/issues/5031)). |
| `Anchor.toml` `[scripts]` | `cargo test` | `cargo test --locked` | Fails loudly instead of silently re-resolving the pinned graph. |
| `.github/workflows/ci.yml` | — | added | `anchor test` on every push and weekly, to catch upstream breakage early. |
| `init-program-id.rhai` | `anchor init` creates the keypair itself | cargo-generate hook (removed after generation) | Gives each project its own program ID instead of a shared placeholder. |
| `README.md` | — | added | This file. |

Everything else (program code, tests, `rust-toolchain.toml` with Rust 1.89, `.gitignore`,
release profile) is untouched `anchor init` output.

**Don't run a blanket `cargo update`**: it re-resolves Agave crates and litesvm 0.15 stops
compiling. Update specific crates with `cargo update -p <crate>`.

## Updating Anchor

1. Read the release notes' *Recommended Solana Version*:
   <https://www.anchor-lang.com/docs/updates/release-notes>.
2. Run `anchor init` with the new version and diff its output against this template to pick
   up upstream changes; drop any fix above that upstream now ships.
3. Update `anchor_version`/`solana_version` in `Anchor.toml`, the `=x.y.z` pin in
   `programs/*/Cargo.toml` and `ANCHOR_VERSION`/`SOLANA_VERSION` in
   `.github/workflows/ci.yml`; then `cargo update -p anchor-lang --precise x.y.z` and
   `anchor test`.
