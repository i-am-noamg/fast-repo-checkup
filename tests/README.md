# Integration tests

- [`cli_integration.rs`](cli_integration.rs) — temp git repo, real `git` and
  `fast-repo-checkup` binary (`CARGO_BIN_EXE_fast_repo_checkup`). Needs a normal
  environment where `git init` can create `.git/hooks` (some sandboxes block
  that).

CI runs these tests on **Ubuntu, macOS, and Windows** (`cargo test --locked` in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml)).

Unit tests live next to the code under `src/` (for example `git::tests`,
`alerts::tests`, `report::tests`).
