use crate::metrics::{self, ScanOptions};
use crate::model::{AlertHint, AlertSeverity, MetricId, MetricResult, ScanReport};
use crate::report;

const OVERLAP_TOP_FILES: usize = 5;
const BUS_FACTOR_DOMINANCE: f64 = 0.60;
const FIREFIGHTING_WARN_PER_YEAR: u64 = 8;

/// Derive simple leadership hints from computed metrics.
pub fn compute_alerts(
    metrics: &[MetricResult],
    recent_since: &str,
    opts: &ScanOptions<'_>,
) -> Vec<AlertHint> {
    let mut alerts = Vec::new();

    let churn_files = file_keys(metrics, MetricId::Churn, OVERLAP_TOP_FILES);
    let bug_files = file_keys(metrics, MetricId::BugHotspots, OVERLAP_TOP_FILES);
    if !churn_files.is_empty() && !bug_files.is_empty() {
        let overlap: Vec<_> = churn_files
            .iter()
            .filter(|f| bug_files.contains(*f))
            .cloned()
            .collect();
        if !overlap.is_empty() {
            alerts.push(AlertHint {
                severity: AlertSeverity::High,
                code: "churn_and_bug_overlap".to_string(),
                message: "Files appear in both high churn and bug hotspots.".to_string(),
                evidence: Some(overlap.join(", ")),
            });
        }
    }

    if let Some((top_name, top_n, total)) = top_author_share(metrics, MetricId::BusFactor) {
        if total > 0 {
            let ratio = top_n as f64 / total as f64;
            if ratio >= BUS_FACTOR_DOMINANCE {
                alerts.push(AlertHint {
                    severity: AlertSeverity::Warning,
                    code: "bus_factor_dominance".to_string(),
                    message: format!(
                        "Top contributor authored {:.0}% of commits (full history on HEAD).",
                        ratio * 100.0
                    ),
                    evidence: Some(format!("{top_name} ({top_n}/{total})")),
                });
            }
        }

        if let Ok(recent) = metrics::bus_factor_recent_authors(opts) {
            let recent_names: std::collections::HashSet<_> =
                recent.iter().map(|(n, _)| n.as_str()).collect();
            if !recent_names.contains(top_name.as_str()) {
                alerts.push(AlertHint {
                    severity: AlertSeverity::Warning,
                    code: "departed_top_contributor".to_string(),
                    message: format!(
                        "Top contributor \"{top_name}\" has no commits since {recent_since} on HEAD."
                    ),
                    evidence: Some(format!("recent_since={recent_since}")),
                });
            }
        }
    }

    if let Some(n) = scalar(metrics, MetricId::Firefighting) {
        if n >= FIREFIGHTING_WARN_PER_YEAR {
            alerts.push(AlertHint {
                severity: AlertSeverity::Warning,
                code: "firefighting_frequency".to_string(),
                message: format!(
                    "Many revert/hotfix-style commits in window (>= {FIREFIGHTING_WARN_PER_YEAR})."
                ),
                evidence: Some(format!("count={n}")),
            });
        }
    }

    if let Some(msg) = delivery_drop_hint(metrics) {
        alerts.push(AlertHint {
            severity: AlertSeverity::Warning,
            code: "delivery_pace_drop".to_string(),
            message: msg,
            evidence: None,
        });
    }

    alerts
}

pub fn build_report(
    repo: String,
    since: String,
    recent_since: String,
    source_dirs: Vec<String>,
    metrics: Vec<MetricResult>,
    opts: &ScanOptions<'_>,
) -> ScanReport {
    let alerts = compute_alerts(&metrics, &recent_since, opts);
    ScanReport {
        warnings: report::source_dir_warnings(&metrics, &source_dirs),
        repo,
        since,
        recent_since,
        source_dirs,
        metrics,
        alerts,
    }
}

fn file_keys(
    metrics: &[MetricResult],
    id: MetricId,
    top: usize,
) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let Some(m) = metrics.iter().find(|m| m.id == id) else {
        return set;
    };
    let Some(rows) = &m.rows else {
        return set;
    };
    for r in rows.iter().take(top) {
        set.insert(r.key.clone());
    }
    set
}

fn scalar(metrics: &[MetricResult], id: MetricId) -> Option<u64> {
    metrics.iter().find(|m| m.id == id)?.scalar
}

/// Returns (author_name, top_commits, total_commits) for bus factor metric rows.
fn top_author_share(metrics: &[MetricResult], id: MetricId) -> Option<(String, u64, u64)> {
    let m = metrics.iter().find(|m| m.id == id)?;
    let rows = m.rows.as_ref()?;
    let top = rows.first()?;
    let total = m
        .scalar
        .unwrap_or_else(|| rows.iter().map(|r| r.value).sum());
    Some((top.key.clone(), top.value, total))
}

fn delivery_drop_hint(metrics: &[MetricResult]) -> Option<String> {
    delivery_drop_hint_with_now(metrics, &current_utc_year_month())
}

/// Compare the latest complete month to the prior three months. Skips the
/// in-progress current month so a few commits on day 3 do not look like a cliff.
fn delivery_drop_hint_with_now(metrics: &[MetricResult], current_ym: &str) -> Option<String> {
    let m = metrics.iter().find(|m| m.id == MetricId::DeliveryPace)?;
    let rows = m.rows.as_ref()?;
    let skip_trailing = usize::from(rows.last().is_some_and(|r| r.key == current_ym));
    let need = 4 + skip_trailing;
    if rows.len() < need {
        return None;
    }
    let eval_idx = rows.len() - 1 - skip_trailing;
    let last = rows[eval_idx].value;
    let prev: u64 = rows[eval_idx - 3..eval_idx]
        .iter()
        .map(|r| r.value)
        .sum();
    let prev_avg = prev as f64 / 3.0;
    if prev_avg < 1.0 {
        return None;
    }
    if (last as f64) < prev_avg * 0.5 {
        Some(format!(
            "Last month commits ({last}) are below half the trailing 3-month average ({prev_avg:.1})."
        ))
    } else {
        None
    }
}

fn current_utc_year_month() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86_400;
    let (year, month, _) = utc_days_to_ymd(days);
    format!("{year:04}-{month:02}")
}

/// Days since 1970-01-01 (UTC) to (year, month, day).
fn utc_days_to_ymd(days: i64) -> (i32, u32, u32) {
    let mut z = days + 719_468;
    let era = z.div_euclid(146_097);
    z -= era * 146_097;
    let yoe = (z - z / 1460 + z / 36524 - z / 146096) / 365;
    let y = yoe + era * 400;
    let doy = z - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MetricRow;
    use crate::source_dirs::SourceDirMatcher;

    fn test_opts(source_matcher: &SourceDirMatcher) -> ScanOptions<'_> {
        ScanOptions {
            repo: std::path::Path::new("."),
            since: "1 year ago",
            recent_since: "6 months ago",
            source_dirs: &[],
            source_matcher,
            top: 20,
        }
    }

    #[test]
    fn overlap_alert() {
        let source_matcher = SourceDirMatcher::new(&[]).unwrap();
        let opts = test_opts(&source_matcher);
        let metrics = vec![
            MetricResult {
                id: MetricId::Churn,
                label: "".into(),
                summary: "".into(),
                rows: Some(vec![MetricRow {
                    key: "a.rs".into(),
                    value: 10,
                    extra: None,
                }]),
                scalar: None,
            },
            MetricResult {
                id: MetricId::BugHotspots,
                label: "".into(),
                summary: "".into(),
                rows: Some(vec![MetricRow {
                    key: "a.rs".into(),
                    value: 2,
                    extra: None,
                }]),
                scalar: None,
            },
        ];
        let a = compute_alerts(&metrics, "6 months ago", &opts);
        assert!(a.iter().any(|x| x.code == "churn_and_bug_overlap"));
    }

    fn delivery_pace_metrics(rows: Vec<MetricRow>) -> Vec<MetricResult> {
        vec![MetricResult {
            id: MetricId::DeliveryPace,
            label: "".into(),
            summary: "".into(),
            rows: Some(rows),
            scalar: None,
        }]
    }

    #[test]
    fn delivery_pace_drop_ignores_in_progress_current_month() {
        let metrics = delivery_pace_metrics(vec![
            MetricRow {
                key: "2026-03".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-04".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-05".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-06".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-07".into(),
                value: 2,
                extra: None,
            },
        ]);
        assert!(delivery_drop_hint_with_now(&metrics, "2026-07").is_none());
    }

    #[test]
    fn delivery_pace_drop_flags_last_complete_month() {
        let metrics = delivery_pace_metrics(vec![
            MetricRow {
                key: "2026-03".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-04".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-05".into(),
                value: 30,
                extra: None,
            },
            MetricRow {
                key: "2026-06".into(),
                value: 5,
                extra: None,
            },
            MetricRow {
                key: "2026-07".into(),
                value: 1,
                extra: None,
            },
        ]);
        assert!(delivery_drop_hint_with_now(&metrics, "2026-07").is_some());
    }
}
