# Security Policy

## Supported versions

Security fixes are applied to the latest release on the default branch (`main`).

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Email the maintainer privately (see the GitHub profile for contact) with:

- A description of the issue and impact
- Steps to reproduce
- Affected versions or commits

We aim to acknowledge reports within a few business days.

## Threat model

`fast-repo-checkup` is a **local, read-only CLI** that runs `git` subprocesses to analyze commit history. It does not open network connections, write to the target repository, or execute shell pipelines.

### Intended use

- Run on repositories you trust or control (your laptop, known client repos).
- Use output as engineering diagnostics, not as an authorization or security boundary.

### Out of scope / higher risk use

- Scanning **untrusted** repositories (malicious commit messages, path names, or author strings may appear in terminal or JSON output).
- Running in **shared CI** on arbitrary fork PRs without isolation — treat JSON artifacts and logs as potentially containing hostile text.
- Pointing `--repo` at sensitive directories you do not intend to analyze.

## Hardening measures

| Area | Mitigation |
|------|------------|
| Command injection | No shell; `git` invoked with explicit argument list |
| Environment hijack | `GIT_*`, `LD_PRELOAD`, `DYLD_*`, and `FAST_REPO_CHECKUP_*` are not passed to git child processes |
| Git binary | Override with `FAST_REPO_CHECKUP_GIT` (single-line path only) |
| Pathspec abuse | `--source-dir` rejects user-supplied pathspec magic (`:`), `..`, absolute paths, and `-` prefixes; globs are wrapped as `:(glob)…` by the CLI |
| Output safety | Git-derived strings are sanitized before table/JSON output (ANSI/control stripping) |
| Error leakage | Git stderr is omitted from default errors; set `FAST_REPO_CHECKUP_VERBOSE=1` for details |
| Supply chain | `Cargo.lock` committed; CI runs `cargo test --locked`, RustSec audit, and `cargo deny` |

## Dependency and CI security

- [`.github/workflows/ci.yml`](.github/workflows/ci.yml) uses minimal workflow permissions (`contents: read`).
- [`.github/workflows/security.yml`](.github/workflows/security.yml) runs a weekly RustSec audit for newly disclosed advisories.
- [`.github/dependabot.yml`](.github/dependabot.yml) proposes dependency updates.
- [`deny.toml`](deny.toml) enforces license, advisory, and source policies in CI.
- Rust caches are job-scoped, exclude installed binaries, save only from trusted branch runs, and are not used by the token-bearing release publish job.

## Safe development practices

- Do not commit secrets (`.env`, keys, tokens). See [`.gitignore`](.gitignore).
- Run `cargo deny check` and `cargo audit` locally before release.
- When adding metrics that invoke git, keep stdin closed and pass `-C <repo>` explicitly (see [`docs/architecture.md`](docs/architecture.md)).
