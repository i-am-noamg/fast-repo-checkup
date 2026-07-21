# fast-repo-checkup

Inspired by [The Git Commands I Run Before Reading Any Code](https://piechowski.io/post/git-commands-before-reading-code/).

`fast-repo-checkup` is a Rust CLI that maps code churn, ownership concentration, bug
hotspots, delivery pace, and firefighting patterns before you read an unfamiliar codebase.
It also adds lightweight alert hints for signals worth investigating.

[docs/blogpost.md](docs/blogpost.md) is a markdown version of the original blogpost.

## Quick start

Requirements: **git** on `PATH` and a repo with **at least one commit**. Cargo installs
also require a Rust toolchain; prebuilt binaries are available below.

Install the latest published release from [crates.io](https://crates.io/crates/fast-repo-checkup):

```bash
cargo install fast-repo-checkup --locked
fast-repo-checkup scan --repo /path/to/repo --source-dir src
```

For local development from a checkout:

```bash
cargo build
cargo run -- scan --repo . --source-dir src
```

JSON:

```bash
cargo run -- scan --repo . --format json
```

Install directly from the repository instead:

```bash
cargo install --git https://github.com/i-am-noamg/fast-repo-checkup --locked
fast-repo-checkup scan --repo /path/to/repo
```

Prebuilt binaries are also published on [GitHub Releases](https://github.com/i-am-noamg/fast-repo-checkup/releases) for Linux, macOS, and Windows.

## Interpreting the results

These are historical signals, not definitive code-quality scores. The results depend
on commit history and message quality:

- churn counts commits that touch files; it does not measure lines changed
- bug hotspots match bug-related words in commit messages; they are not production
  incident data
- ownership is based on commit counts, and squash merges can hide contributors
- delivery pace and firefighting patterns describe Git history, not the whole delivery process

Use the report to decide where to look next, then validate the signal in the code and
with the people who maintain it.

## Representative output

```text
== Alerts ==
[High] churn_and_bug_overlap — Files appear in both high churn and bug hotspots.
[Warning] bus_factor_dominance — Top contributor dominates the commit history.
```

The exact alerts and evidence vary by repository, time window, and source directories.



## Documentation

- [docs/README.md](docs/README.md) — doc index for humans and agents
- [docs/cli-usage.md](docs/cli-usage.md) — commands, flags, install, tests
- [docs/architecture.md](docs/architecture.md) — Rust layout and guardrails
- [docs/git-metrics.md](docs/git-metrics.md) — what each signal means



## Contributing

Local checks (match CI):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

[`rust-toolchain.toml`](rust-toolchain.toml) pins stable with `rustfmt` and `clippy` for local dev. MSRV is `rust-version` in [`Cargo.toml`](Cargo.toml) (currently 1.85+).

CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) on push/PR to `main`/`master`:

- **fmt** — formatting check (Ubuntu)
- **clippy** — lints with `-D warnings`, `--locked` (Ubuntu)
- **test** — `cargo test --locked` on Ubuntu, macOS, and Windows
- **msrv** — `cargo test --locked` on Rust 1.85
- **install-smoke** — `cargo install --path . --locked` and a binary smoke test
- **audit** — `rustsec/audit-check` for dependency advisories
- **deny** — `cargo deny check all` for licenses and banned sources

Rust dependency caches are job-scoped, exclude installed binaries, and are saved only by trusted branch runs. The release publishing job does not use a Rust cache.

The scheduled security workflow (`.github/workflows/security.yml`) runs a weekly RustSec audit to catch advisories disclosed after the last code change.

Dependabot (`.github/dependabot.yml`) opens weekly Cargo and monthly GitHub Actions update PRs.

Security policy: [`SECURITY.md`](SECURITY.md).

## Releases

Maintainers cut releases manually from GitHub Actions via `.github/workflows/release.yml`.
Each release is published to [crates.io](https://crates.io/crates/fast-repo-checkup) and
[GitHub Releases](https://github.com/i-am-noamg/fast-repo-checkup/releases).
Create a protected GitHub Environment named `release`, add at least one required reviewer,
and store the crate-scoped `CARGO_REGISTRY_TOKEN` there. Run each dispatch with
`dry_run=true` first when validating the version, CI gates, and packaging before approving
a live publish in the protected `release` environment.
