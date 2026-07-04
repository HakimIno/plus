# plus

`plus` is a Rust workflow doctor, smart cleaner, setup optimizer, and Cargo launcher.

It does not replace Cargo. Cargo remains the build engine. `plus` helps humans keep Rust
projects fast, clean, and correctly configured.

## Install

From a local checkout:

```sh
cargo install --path .
```

From GitHub, once published:

```sh
cargo install --git https://github.com/your-name/plus
```

## Daily Use

```sh
plus doctor
plus init
plus size --deep
plus clean
plus clean --apply
plus setup
plus setup --write
plus dev
plus test
plus doctor --json
plus size --deep --json
plus clean --json
```

## What Makes It Different

Rust already has excellent tools: Cargo, bacon, cargo-nextest, sccache, mold/lld,
and cargo-cache. The problem is that developers have to know which tool to use,
when to use it, and how to configure it.

`plus` is the decision layer:

- diagnoses missing tools and slow build setup
- checks whether installed tools are actually usable
- explains why `target/` is large
- previews cleanup before deleting anything
- creates local Cargo speed config without overwriting existing config
- chooses a good dev/test command based on installed tools

## Commands

### `plus doctor`

Inspects the current Rust project, common workflow tools, Cargo config, target size,
and prints recommendations.

It also runs a real `sccache rustc -vV` health check. If `sccache` is installed but
not usable, `plus doctor` reports that separately from a missing install.

For automation:

```sh
plus doctor --json
```

### `plus init`

Creates a project-local `plus.toml` with useful defaults. It does not overwrite an
existing file unless `--force` is passed.

```sh
plus init
plus init --force
```

### `plus size --deep`

Shows which parts of `target/` are taking space:

```text
target: 6.6 GiB
  debug          6.6 GiB
  incremental    564.3 MiB

Deep buckets:
     4.0 GiB  target/debug/deps
     1.3 GiB  target/debug/build
   564.3 MiB  target/debug/incremental
```

For automation:

```sh
plus size --deep --json
```

### `plus clean`

Defaults to a dry run. It does not delete files unless `--apply` is passed.

```sh
plus clean
plus clean --apply
plus clean --json
plus clean --deep --apply
plus clean --nuclear --yes
```

`--nuclear --yes` runs `cargo clean`. The extra `--yes` is intentional so a full
cleanup cannot happen by accident.

`plus clean --json` prints the cleanup plan without deleting anything unless
`--apply` is also passed.

### `plus setup --write`

Creates `.cargo/config.toml` if it does not already exist. It only enables settings
for tools already installed and usable, and comments out settings for missing or
broken tools.

### `plus dev`

Uses the best available dev loop:

- `bacon` when available
- otherwise `cargo watch -x check` when available
- otherwise `cargo check`

### `plus test`

Uses `cargo nextest run` when available, otherwise falls back to `cargo test`.

## Project Config

Create `plus.toml` in a Rust project manually, or run `plus init`:

```toml
[commands]
app = "cargo run -p app --bin app"
quick = "cargo check --workspace"

[clean]
max_target_size = "5GiB"

[dev]
prefer = "bacon"

[tools]
prefer_sccache = true
prefer_fast_linker = true
prefer_nextest = true
```

Then run:

```sh
plus run app
plus run quick
```

## Status

This is an early implementation with production-safety basics in place:

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo package --locked
```

The intended scope is:

- not a Cargo replacement
- not a compiler
- not a dependency resolver
- a practical workflow assistant for local Rust development
