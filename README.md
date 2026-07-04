# plus

`plus` is a Rust workflow doctor, smart cleaner, setup optimizer, and Cargo launcher.

It does not replace Cargo. Cargo remains the build engine. `plus` helps humans keep Rust
projects fast, clean, and correctly configured.

## Install

### Prerequisites

You need Rust and Cargo installed first:

```sh
rustc --version
cargo --version
```

Recommended installer:

- Linux/macOS: <https://rustup.rs>
- Windows: <https://rustup.rs> or `winget install Rustlang.Rustup`

### From GitHub Source

This works on Linux, macOS, and Windows:

```sh
cargo install --git https://github.com/HakimIno/plus
```

### From A Local Checkout

```sh
git clone git@github.com:HakimIno/plus.git
cd plus
cargo install --path .
```

This installs two executables:

```text
plus
cargo-plus
```

So every command can be run in either style:

```sh
plus doctor
cargo plus doctor
```

### From Release Archives

Tagged releases build archives for Linux, macOS, and Windows.

Linux:

```sh
tar -xzf plus-x86_64-unknown-linux-gnu.tar.gz
cd plus-x86_64-unknown-linux-gnu
chmod +x plus cargo-plus
./plus doctor
```

macOS Intel:

```sh
tar -xzf plus-x86_64-apple-darwin.tar.gz
cd plus-x86_64-apple-darwin
chmod +x plus cargo-plus
./plus doctor
```

macOS Apple Silicon:

```sh
tar -xzf plus-aarch64-apple-darwin.tar.gz
cd plus-aarch64-apple-darwin
chmod +x plus cargo-plus
./plus doctor
```

Windows PowerShell:

```powershell
Expand-Archive .\plus-x86_64-pc-windows-msvc.zip
cd .\plus-x86_64-pc-windows-msvc\plus-x86_64-pc-windows-msvc
.\plus.exe doctor
```

To use release binaries from anywhere, move them into a directory on your `PATH`.

Linux/macOS example:

```sh
mkdir -p ~/.local/bin
cp plus cargo-plus ~/.local/bin/
```

Windows example:

```powershell
mkdir $HOME\bin
copy .\plus.exe $HOME\bin\
copy .\cargo-plus.exe $HOME\bin\
```

Then add `$HOME\bin` to your user `Path` environment variable.

### PATH Troubleshooting

If `cargo install` succeeds but `plus` is not found, make sure Cargo's bin directory
is on your `PATH`.

Linux/macOS:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Windows PowerShell:

```powershell
$env:Path += ";$HOME\.cargo\bin"
```

Permanent PATH setup depends on your shell or Windows user environment settings.

### Optional Tools

`plus` works without these, but it can use them when available:

- `sccache` for compiler caching
- `mold` or `lld` for faster linking
- `bacon` or `cargo-watch` for a better dev loop
- `cargo-nextest` for faster tests

Run:

```sh
plus doctor
plus setup
```

Platform hints:

Linux Fedora:

```sh
sudo dnf install -y sccache mold lld
cargo install bacon cargo-nextest
```

Linux Ubuntu/Debian:

```sh
sudo apt install -y sccache lld
cargo install bacon cargo-nextest
```

macOS:

```sh
brew install sccache mold
cargo install bacon cargo-nextest
```

Windows:

```powershell
winget install Mozilla.sccache
cargo install bacon cargo-nextest
```

## Daily Use

```sh
plus doctor
cargo plus doctor
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

CI runs on Linux, macOS, and Windows. Tagged releases (`v*`) build archives for:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

The intended scope is:

- not a Cargo replacement
- not a compiler
- not a dependency resolver
- a practical workflow assistant for local Rust development
