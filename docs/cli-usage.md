# CLI usage

Binary name: `fast-repo-checkup` (same as the Rust package).

Canonical metric definitions: [`docs/blogpost.md`](blogpost.md).

## Requirements

- `git` on `PATH`
- A local clone (bare repos work if `git -C <path>` accepts them)
- At least one commit in the repository (empty `git init` with no commits is rejected with a clear error)

## Commands

### `scan`

Runs all five metrics and prints alert hints.

```bash
cargo run -- scan --repo . --source-dir src
```

Common flags:

- `--repo <path>` — repository root (default: `.`)
- `--source-dir <path>` — repeatable; scopes churn and bug_hotspots (blog: run from `src/` or `app/`). Globs supported, e.g. `services/*/src`.
- `--since <git-date>` — all metrics (default: `1 year ago`); bus factor and bug hotspots use full history when `--full-history` is set
- `--recent-since <git-date>` — bus-factor departed-contributor alerts (default: `6 months ago`; should be ≤ `--since`)
- `--full-history` — blog-faithful full history for bus factor and bug hotspots
- `--top <n>` — max rows for file/author tables (default: `20`)
- `--format table|json` — output (default: `table`)
- `--no-color` — disable ANSI colors in table output (also respects `NO_COLOR`)

When `--source-dir` is omitted, file metrics scan the whole repo and a warning is printed at the start of the output.

JSON example:

```bash
cargo run -- scan --format json --repo /path/to/repo --source-dir src --source-dir apps
```

### `metrics`

Runs one metric by id or alias:

- `churn` — high-churn files (`--since`, `--source-dir`)
- `bus_factor` — contributor shortlog on `HEAD` (`--since`; `--recent-since` for alerts; `--full-history` for blog mode)
- `bug_hotspots` — commits matching fix|bug|broken (`--since`, `--source-dir`; `--full-history` for blog mode)
- `delivery_pace` — commits per `YYYY-MM` within `--since` (default: `1 year ago`)
- `firefighting` — oneline subjects matching revert/hotfix/emergency/rollback (`--since`)

```bash
cargo run -- metrics churn --repo . --source-dir src --since "1 year ago"
cargo run -- metrics bus_factor --repo . --recent-since "6 months ago"
```

### `explain`

Prints the blogpost command and the CLI's git equivalent.

```bash
cargo run -- explain bus_factor
```

## Install

Install the latest published release from crates.io:

```bash
cargo install fast-repo-checkup --locked
```

Or install from the GitHub repository while developing an unreleased change:

```bash
cargo install --git https://github.com/i-am-noamg/fast-repo-checkup --locked
```

Prebuilt binaries for Linux, macOS, and Windows are available from [GitHub Releases](https://github.com/i-am-noamg/fast-repo-checkup/releases).

## Tests

```bash
cargo test --locked
```

See [`tests/README.md`](../tests/README.md): integration tests build a temporary
git repository and run the `fast-repo-checkup` binary (`CARGO_BIN_EXE_fast_repo_checkup`).
Requires **git** on `PATH`.

CI runs the same test suite on Ubuntu, macOS, and Windows, plus fmt, clippy, MSRV,
install smoke, and dependency audit — see [`architecture.md`](architecture.md#testing).

## Environment variables

| Variable | Purpose |
|----------|---------|
| `FAST_REPO_CHECKUP_GIT` | Path to the `git` executable (single line; not passed to child env) |
| `FAST_REPO_CHECKUP_VERBOSE` | Set to `1`, `true`, or `yes` to print git stderr on failures |

See [`SECURITY.md`](../SECURITY.md) for the threat model and safe usage in CI.
