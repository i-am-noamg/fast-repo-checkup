# Git Health Metrics

Canonical source: [`docs/blogpost.md`](blogpost.md) — five git commands run before reading code.

## 1) High-churn files

Run from source directories (`src/`, `app/`), not the repo root — lockfiles and generated files dominate otherwise.

Command:

```bash
git log --format=format: --name-only --since="1 year ago" | sort | uniq -c | sort -nr | head -20
```

CLI: `--source-dir src` (repeatable), `--since "1 year ago"`, `--top 20`.

Why it matters:

- High churn can indicate unstable architecture or concentrated risk.
- Cross-check with bug hotspots to find files that change a lot and break a lot.

## 2) Bus factor / ownership concentration

Default window (`--since`, default `1 year ago`):

```bash
git shortlog -sn --no-merges --since="1 year ago"
```

Blog-faithful full history (`--full-history`):

```bash
git shortlog -sn --no-merges
```

Secondary window for departed-contributor check:

```bash
git shortlog -sn --no-merges --since="6 months ago"
```

CLI: `--since "1 year ago"`, `--recent-since "6 months ago"`, `--full-history` for blog mode. The tool passes `HEAD` explicitly so `shortlog` does not read from stdin (empty under a closed stdin in subprocesses).

Why it matters:

- If one person dominates the analysis window, a departure creates knowledge risk.
- If the top contributor from the main window is absent in the recent window, flag it immediately.

## 3) Bug hotspots

Same source-dir scoping as churn. Default window (`--since`); blog uses full history (`--full-history`):

```bash
git log -i -E --grep="fix|bug|broken" --name-only --format='' --since="1 year ago"
```

CLI: `--source-dir src` (repeatable), `--since "1 year ago"`, `--top 20`, `--full-history` for blog mode.

Why it matters:

- Frequent bug fixes in the same files are a strong quality signal.
- Compare against churn hotspots; overlap is highest risk.

## 4) Delivery pace

Commit volume by month. The blog uses full history:

```bash
git log --format='%ad' --date=format:'%Y-%m' | sort | uniq -c
```

CLI: `--since "1 year ago"` (default) scopes the log to a rolling window so output stays readable.

Why it matters:

- Sharp drops can indicate staffing changes or stalled delivery.
- Spiky output can indicate batched releases instead of steady shipping.

## 5) Firefighting frequency

```bash
git log --oneline --since="1 year ago" | grep -iE 'revert|hotfix|emergency|rollback'
```

CLI: `--since "1 year ago"`.

Why it matters:

- Frequent reverts indicate fragile deployment or low confidence in changes.

## Per-metric `--since` rules

| Metric | `--since` | `--recent-since` | `--full-history` | `--source-dir` |
|--------|-----------|------------------|------------------|----------------|
| churn | yes (default `1 year ago`) | — | — | yes |
| bus_factor | yes (default `1 year ago`) | yes (alerts) | blog mode | no |
| bug_hotspots | yes (default `1 year ago`) | — | blog mode | yes |
| delivery_pace | yes (default `1 year ago`) | — | — | no |
| firefighting | yes (default `1 year ago`) | — | — | no |
