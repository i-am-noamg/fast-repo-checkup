//! Integration tests: temp git repo + CLI JSON smoke.

use std::fs;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let st = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("run git");
    assert!(st.success(), "git {:?} failed", args);
}

fn git_with_env(repo: &Path, args: &[&str], env: &[(&str, &str)]) {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo).args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let st = cmd.status().expect("run git");
    assert!(st.success(), "git {:?} failed", args);
}

fn init_fixture_repo(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "fixture@repodragglance.test"],
    );
    git(root, &["config", "user.name", "Fixture"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "// lib\n").unwrap();
    fs::write(root.join("README.md"), "# x\n").unwrap();
    fs::write(root.join("Cargo.lock"), "version = 3\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "init"]);
    fs::write(root.join("src/lib.rs"), "// lib fix bug\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "fix bug in lib"]);
    git(
        root,
        &["commit", "--allow-empty", "-m", "Revert bad deploy"],
    );
}

fn init_monorepo_fixture(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "fixture@repodragglance.test"],
    );
    git(root, &["config", "user.name", "Fixture"]);
    fs::create_dir_all(root.join("services/foo/src")).unwrap();
    fs::create_dir_all(root.join("services/bar/src")).unwrap();
    fs::write(root.join("services/foo/src/lib.rs"), "// foo\n").unwrap();
    fs::write(root.join("services/bar/src/lib.rs"), "// bar\n").unwrap();
    fs::write(root.join("Cargo.lock"), "version = 3\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "init"]);
    fs::write(root.join("services/foo/src/lib.rs"), "// foo fix bug\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "fix bug in foo"]);
}

fn prior_lead_env() -> [(&'static str, &'static str); 6] {
    [
        ("GIT_AUTHOR_NAME", "PriorLead"),
        ("GIT_AUTHOR_EMAIL", "prior@repodragglance.test"),
        ("GIT_COMMITTER_NAME", "PriorLead"),
        ("GIT_COMMITTER_EMAIL", "prior@repodragglance.test"),
        ("GIT_AUTHOR_DATE", "2025-11-15T12:00:00"),
        ("GIT_COMMITTER_DATE", "2025-11-15T12:00:00"),
    ]
}

fn init_windowed_departed_repo(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "fixture@repodragglance.test"],
    );
    git(root, &["config", "user.name", "Fixture"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "// v1\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "prior 1"], &prior_lead_env());
    fs::write(root.join("src/lib.rs"), "// v2\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "prior 2"], &prior_lead_env());
    fs::write(root.join("src/lib.rs"), "// v3\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "prior 3"], &prior_lead_env());
    fs::write(root.join("src/lib.rs"), "// recent\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "recent work"]);
}

fn old_author_env() -> [(&'static str, &'static str); 6] {
    [
        ("GIT_AUTHOR_NAME", "OldAuthor"),
        ("GIT_AUTHOR_EMAIL", "old@repodragglance.test"),
        ("GIT_COMMITTER_NAME", "OldAuthor"),
        ("GIT_COMMITTER_EMAIL", "old@repodragglance.test"),
        ("GIT_AUTHOR_DATE", "2020-01-01T12:00:00"),
        ("GIT_COMMITTER_DATE", "2020-01-01T12:00:00"),
    ]
}

fn init_departed_author_repo(root: &Path) {
    fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "fixture@repodragglance.test"],
    );
    git(root, &["config", "user.name", "Fixture"]);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "// v1\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "old 1"], &old_author_env());
    fs::write(root.join("src/lib.rs"), "// v2\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "old 2"], &old_author_env());
    fs::write(root.join("src/lib.rs"), "// v3\n").unwrap();
    git(root, &["add", "."]);
    git_with_env(root, &["commit", "-m", "old 3"], &old_author_env());
    fs::write(root.join("src/lib.rs"), "// recent\n").unwrap();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "recent work"]);
}

fn repo_drag_glance_bin() -> std::path::PathBuf {
    if let Some(p) = std::env::var_os("CARGO_BIN_EXE_repo_drag_glance") {
        return std::path::PathBuf::from(p);
    }
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target"));
    #[cfg(windows)]
    let name = "repo-drag-glance.exe";
    #[cfg(not(windows))]
    let name = "repo-drag-glance";
    target.join("debug").join(name)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(repo_drag_glance_bin())
        .args(args)
        .output()
        .expect("run repo-drag-glance")
}

#[test]
fn scan_json_has_metrics_and_alerts() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "scan",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);

    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(v.get("metrics").and_then(|m| m.as_array()).is_some());
    assert!(v.get("alerts").and_then(|a| a.as_array()).is_some());
    let metrics = v["metrics"].as_array().unwrap();
    assert_eq!(metrics.len(), 5);
    let ids: Vec<_> = metrics
        .iter()
        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
        .collect();
    assert!(ids.contains(&"churn"));
    assert!(ids.contains(&"firefighting"));

    let warnings = v["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| {
            w.as_str()
                .is_some_and(|s| s.contains("No --source-dir set"))
        }),
        "expected source-dir warning at start of output"
    );
}

#[test]
fn metrics_single_churn_json() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "metrics",
        "churn",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);

    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(v["metrics"].as_array().map(|a| a.len()), Some(1));
}

#[test]
fn scan_fails_clearly_on_repo_with_no_commits() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    fs::create_dir_all(repo).unwrap();
    git(repo, &["init"]);

    let out = run_cli(&[
        "scan",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1 year ago",
        "--format",
        "json",
    ]);

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no commits") || stderr.contains("empty history"),
        "expected empty-repo hint, got: {stderr}"
    );
}

#[test]
fn source_dir_excludes_root_lockfile_from_churn() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "metrics",
        "churn",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "src",
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = v["metrics"][0]["rows"].as_array().unwrap();
    let keys: Vec<_> = rows
        .iter()
        .filter_map(|r| r.get("file").and_then(|k| k.as_str()))
        .collect();
    assert!(keys.iter().any(|k| k.starts_with("src/")));
    assert!(!keys.contains(&"Cargo.lock"));
}

#[test]
fn bug_hotspots_finds_fix_commit_in_since_window() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "metrics",
        "bug_hotspots",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "src",
        "--format",
        "json",
    ]);
    assert!(out.status.success());

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = v["metrics"][0]["rows"].as_array().unwrap();
    assert!(
        rows.iter()
            .any(|r| r.get("file").and_then(|k| k.as_str()) == Some("src/lib.rs")),
        "expected src/lib.rs in bug hotspots"
    );
}

#[test]
fn source_dir_set_suppresses_warning() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "scan",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "src",
        "--format",
        "json",
    ]);
    assert!(out.status.success());

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let warnings = v.get("warnings").and_then(|w| w.as_array());
    assert!(
        warnings.is_none() || warnings.is_some_and(|w| w.is_empty()),
        "expected no source-dir warning when --source-dir is set"
    );
}

#[test]
fn bus_factor_departed_top_contributor_alert() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);

    let out = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--full-history",
        "--recent-since",
        "1 day ago",
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let alerts = v["alerts"].as_array().unwrap();
    assert!(
        alerts.iter().any(|a| {
            a.get("code").and_then(|c| c.as_str()) == Some("departed_top_contributor")
        }),
        "expected departed_top_contributor alert"
    );
}

#[test]
fn bus_factor_windowed_departed_top_contributor_alert() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_windowed_departed_repo(repo);

    let out = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1 year ago",
        "--recent-since",
        "6 months ago",
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let alerts = v["alerts"].as_array().unwrap();
    assert!(
        alerts.iter().any(|a| {
            a.get("code").and_then(|c| c.as_str()) == Some("departed_top_contributor")
        }),
        "expected departed_top_contributor alert for windowed bus factor"
    );
}

#[test]
fn bus_factor_respects_since_flag() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);

    let out_narrow = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1 year ago",
        "--format",
        "json",
    ]);
    assert!(out_narrow.status.success());
    let narrow: serde_json::Value = serde_json::from_slice(&out_narrow.stdout).unwrap();
    let narrow_total = narrow["metrics"][0]["scalar"].as_u64().unwrap();

    let out_wide = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);
    assert!(out_wide.status.success());
    let wide: serde_json::Value = serde_json::from_slice(&out_wide.stdout).unwrap();
    let wide_total = wide["metrics"][0]["scalar"].as_u64().unwrap();

    assert_eq!(narrow_total, 1, "narrow since should exclude old commits");
    assert!(
        wide_total > narrow_total,
        "wider --since should include more commits; narrow={narrow_total} wide={wide_total}"
    );
}

#[test]
fn bus_factor_full_history_ignores_since() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);

    let out = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--full-history",
        "--since",
        "1 day ago",
        "--format",
        "json",
    ]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let total = v["metrics"][0]["scalar"].as_u64().unwrap();
    assert_eq!(
        total, 4,
        "bus_factor --full-history should ignore --since; got {total} commits"
    );
    assert_eq!(v["full_history"], true);
}

#[test]
fn bug_hotspots_respects_since_flag() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);
    fs::write(repo.join("src/lib.rs"), "// old bug fix\n").unwrap();
    git(repo, &["add", "."]);
    git_with_env(
        repo,
        &["commit", "-m", "fix ancient bug"],
        &old_author_env(),
    );

    let out_narrow = run_cli(&[
        "metrics",
        "bug_hotspots",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "src",
        "--since",
        "1 year ago",
        "--format",
        "json",
    ]);
    assert!(out_narrow.status.success());
    let narrow: serde_json::Value = serde_json::from_slice(&out_narrow.stdout).unwrap();
    let narrow_rows = narrow["metrics"][0]["rows"].as_array().unwrap().len();

    let out_full = run_cli(&[
        "metrics",
        "bug_hotspots",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "src",
        "--full-history",
        "--format",
        "json",
    ]);
    assert!(out_full.status.success());
    let full: serde_json::Value = serde_json::from_slice(&out_full.stdout).unwrap();
    let full_rows = full["metrics"][0]["rows"].as_array().unwrap().len();

    assert!(
        full_rows >= narrow_rows,
        "full history should include at least as many hotspot files; narrow={narrow_rows} full={full_rows}"
    );
    assert_eq!(full["full_history"], true);
}

#[test]
fn delivery_pace_respects_since_flag() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);

    let out_narrow = run_cli(&[
        "metrics",
        "delivery_pace",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1 day ago",
        "--format",
        "json",
    ]);
    assert!(out_narrow.status.success());
    let narrow: serde_json::Value = serde_json::from_slice(&out_narrow.stdout).unwrap();
    let narrow_scalar = narrow["metrics"][0]["scalar"].as_u64().unwrap();

    let out_wide = run_cli(&[
        "metrics",
        "delivery_pace",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);
    assert!(out_wide.status.success());
    let wide: serde_json::Value = serde_json::from_slice(&out_wide.stdout).unwrap();
    let wide_scalar = wide["metrics"][0]["scalar"].as_u64().unwrap();

    assert!(
        wide_scalar > narrow_scalar,
        "delivery_pace should include more commits with a wider --since; narrow={narrow_scalar} wide={wide_scalar}"
    );
}

#[test]
fn delivery_pace_groups_by_committer_date() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, &["init"]);
    git(repo, &["config", "user.email", "t@t.com"]);
    git(repo, &["config", "user.name", "T"]);

    for m in [
        "2024-01", "2024-06", "2025-01", "2025-06", "2026-01", "2026-06",
    ] {
        git_with_env(
            repo,
            &["commit", "--allow-empty", "-m", &format!("work {m}")],
            &[
                ("GIT_AUTHOR_DATE", &format!("{m}-15 12:00:00")),
                ("GIT_COMMITTER_DATE", "2026-07-03 12:00:00"),
            ],
        );
    }

    let out = run_cli(&[
        "metrics",
        "delivery_pace",
        "--repo",
        repo.to_str().unwrap(),
        "--since",
        "1 year ago",
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = json["metrics"][0]["rows"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "committer date grouping should collapse rebased author months; got {rows:?}"
    );
    assert_eq!(rows[0]["month"], "2026-07");
    let summary = json["metrics"][0]["summary"].as_str().unwrap();
    assert!(
        !summary.contains("6 active months"),
        "summary should not count author-date months: {summary}"
    );
}

#[test]
fn bus_factor_summary_counts_all_contributors() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_departed_author_repo(repo);

    let out = run_cli(&[
        "metrics",
        "bus_factor",
        "--repo",
        repo.to_str().unwrap(),
        "--full-history",
        "--top",
        "1",
        "--format",
        "json",
    ]);
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let summary = v["metrics"][0]["summary"].as_str().unwrap();
    assert!(
        summary.contains("2 contributors"),
        "expected total contributor count in summary, got: {summary}"
    );
    assert!(
        summary.contains("showing top 1"),
        "expected top row cap in summary, got: {summary}"
    );
}

#[test]
fn glob_source_dir_scopes_monorepo_services() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_monorepo_fixture(repo);

    let out = run_cli(&[
        "metrics",
        "churn",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        "services/*/src",
        "--since",
        "1970-01-01",
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows = v["metrics"][0]["rows"].as_array().unwrap();
    let keys: Vec<_> = rows
        .iter()
        .filter_map(|r| r.get("file").and_then(|k| k.as_str()))
        .collect();
    assert!(keys.iter().any(|k| k.starts_with("services/foo/src/")));
    assert!(keys.iter().any(|k| k.starts_with("services/bar/src/")));
    assert!(!keys.contains(&"Cargo.lock"));
}

#[test]
fn rejects_pathspec_source_dir() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    init_fixture_repo(repo);

    let out = run_cli(&[
        "scan",
        "--repo",
        repo.to_str().unwrap(),
        "--source-dir",
        ":(glob)src",
        "--format",
        "json",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("source-dir"),
        "expected validation error, got: {stderr}"
    );
}

#[test]
fn git_failure_hides_stderr_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let not_repo = dir.path().join("not-a-repo");
    fs::create_dir_all(&not_repo).unwrap();

    let out = run_cli(&[
        "scan",
        "--repo",
        not_repo.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not a git repository") || stderr.contains("repository"),
        "expected friendly repo error, got: {stderr}"
    );
    assert!(
        !stderr.contains("fatal:"),
        "git stderr should not be echoed by default: {stderr}"
    );
}
