//! `--source-dir` glob matching and git pathspec conversion.

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// True when the value uses `*` or `?` wildcards.
pub fn is_glob_pattern(value: &str) -> bool {
    value.contains('*') || value.contains('?')
}

/// Git pathspec(s) for a validated source dir (literal or `:(glob)…`).
pub fn to_git_pathspecs(dir: &str) -> Vec<String> {
    if !is_glob_pattern(dir) {
        return vec![dir.to_string()];
    }
    let glob = format!(":(glob){dir}");
    let last = dir
        .rsplit(['/', '\\'])
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(dir);
    if is_glob_pattern(last) {
        vec![glob]
    } else {
        vec![format!("{glob}/**")]
    }
}

fn compile_glob(pattern: &str) -> Result<(), globset::Error> {
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map(|_| ())
}

/// Compile a validated pattern (and its `/**` descendants form) for early error reporting.
pub fn validate_glob_compile(pattern: &str) -> Result<(), globset::Error> {
    let pattern = normalize_path(pattern.trim_end_matches(['/', '\\']));
    compile_glob(&pattern)?;
    compile_glob(&format!("{pattern}/**"))?;
    Ok(())
}

/// Matches file paths against normalized `--source-dir` values (literal prefixes or globs).
pub struct SourceDirMatcher {
    match_all: bool,
    set: GlobSet,
}

impl SourceDirMatcher {
    pub fn new(source_dirs: &[String]) -> Result<Self, globset::Error> {
        if source_dirs.is_empty() {
            return Ok(Self {
                match_all: true,
                set: GlobSetBuilder::new().build()?,
            });
        }
        let mut builder = GlobSetBuilder::new();
        for dir in source_dirs {
            let dir = normalize_path(dir.trim_end_matches(['/', '\\']));
            for pattern in [dir.as_str(), &format!("{dir}/**")] {
                builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
            }
        }
        Ok(Self {
            match_all: false,
            set: builder.build()?,
        })
    }

    pub fn matches(&self, path: &str) -> bool {
        if self.match_all {
            return true;
        }
        let path = path.trim();
        if path.is_empty() {
            return false;
        }
        self.set.is_match(normalize_path(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matcher(dirs: &[&str]) -> SourceDirMatcher {
        let dirs: Vec<String> = dirs.iter().map(|s| (*s).to_string()).collect();
        SourceDirMatcher::new(&dirs).unwrap()
    }

    #[test]
    fn literal_prefix_matches_dir_and_descendants() {
        let m = matcher(&["src"]);
        assert!(m.matches("src"));
        assert!(m.matches("src/lib.rs"));
        assert!(m.matches(r"src\lib.rs"));
        assert!(!m.matches("Cargo.lock"));
    }

    #[test]
    fn glob_matches_multiple_service_trees() {
        let m = matcher(&["services/*/src"]);
        assert!(m.matches("services/foo/src/lib.rs"));
        assert!(m.matches("services/bar/src/main.rs"));
        assert!(m.matches("services/foo/src"));
        assert!(!m.matches("services/foo/lib.rs"));
        assert!(!m.matches("Cargo.lock"));
    }

    #[test]
    fn to_git_pathspecs_wraps_globs() {
        assert_eq!(to_git_pathspecs("src"), vec!["src"]);
        assert_eq!(
            to_git_pathspecs("services/*/src"),
            vec![":(glob)services/*/src/**"]
        );
        assert_eq!(
            to_git_pathspecs("services/*/src/*.rs"),
            vec![":(glob)services/*/src/*.rs"]
        );
    }

    #[test]
    fn empty_dirs_match_all() {
        let m = matcher(&[]);
        assert!(m.matches("anything.rs"));
    }
}
