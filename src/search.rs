use regex::{Regex, RegexBuilder};

use crate::ansi;

/// The active search: compiled pattern plus the direction it was started in.
#[derive(Debug, Clone)]
pub struct Search {
    pub re: Regex,
    pub backwards: bool,
}

/// Compile with smart case: an all-lowercase pattern matches case-insensitively,
/// any uppercase letter makes the search case-sensitive.
pub fn compile(pattern: &str) -> Result<Regex, regex::Error> {
    RegexBuilder::new(pattern)
        .case_insensitive(!pattern.chars().any(|c| c.is_uppercase()))
        .build()
}

/// Find the first matching line at or after/before `start` (inclusive).
/// Lines are matched on their visible text, ignoring ANSI escapes.
pub fn find(lines: &[String], start: usize, backwards: bool, re: &Regex) -> Option<usize> {
    let matches = |i: &usize| re.is_match(&ansi::strip(&lines[*i]));
    if backwards {
        (0..=start.min(lines.len().saturating_sub(1)))
            .rev()
            .find(matches)
    } else {
        (start..lines.len()).find(matches)
    }
}

/// Match spans on one line as (start, end) character indices, end exclusive.
pub fn char_spans(line: &str, re: &Regex) -> Vec<(usize, usize)> {
    if line.is_empty() {
        return Vec::new();
    }
    // Map byte offsets to char indices in one pass.
    let mut byte_to_char = vec![0usize; line.len() + 1];
    for (chars, (bytes, _)) in line.char_indices().enumerate() {
        byte_to_char[bytes] = chars;
    }
    byte_to_char[line.len()] = line.chars().count();

    re.find_iter(line)
        .filter(|m| m.end() > m.start())
        .map(|m| (byte_to_char[m.start()], byte_to_char[m.end()]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn finds_forward_and_backward() {
        let ls = lines(&["alpha", "beta", "gamma", "beta"]);
        let re = compile("beta").unwrap();
        assert_eq!(find(&ls, 0, false, &re), Some(1));
        assert_eq!(find(&ls, 2, false, &re), Some(3));
        assert_eq!(find(&ls, 3, true, &re), Some(3));
        assert_eq!(find(&ls, 2, true, &re), Some(1));
        assert_eq!(find(&ls, 0, true, &re), None);
        assert_eq!(
            compile("delta")
                .ok()
                .and_then(|re| find(&ls, 0, false, &re)),
            None
        );
    }

    #[test]
    fn start_past_end_is_safe() {
        let ls = lines(&["alpha"]);
        let re = compile("alpha").unwrap();
        assert_eq!(find(&ls, 5, false, &re), None);
        assert_eq!(find(&ls, 5, true, &re), Some(0));
        assert_eq!(find(&[], 0, false, &re), None);
    }

    #[test]
    fn spans_are_char_indices() {
        let re = compile("ab").unwrap();
        assert_eq!(char_spans("abxab", &re), vec![(0, 2), (3, 5)]);
        // multi-byte prefix shifts byte offsets but not char indices
        let re = compile("luť").unwrap();
        assert_eq!(char_spans("žluťoučký", &re), vec![(1, 4)]);
    }

    #[test]
    fn smart_case() {
        let ls = lines(&["Alpha", "beta"]);
        assert_eq!(find(&ls, 0, false, &compile("alpha").unwrap()), Some(0));
        assert_eq!(find(&ls, 0, false, &compile("Alpha").unwrap()), Some(0));
        assert_eq!(find(&ls, 0, false, &compile("Beta").unwrap()), None);
    }

    #[test]
    fn matches_ignore_ansi_escapes() {
        let ls = vec!["\x1b[31mcolored\x1b[0m line".to_string()];
        assert_eq!(
            find(&ls, 0, false, &compile("colored line").unwrap()),
            Some(0)
        );
    }

    #[test]
    fn empty_matches_are_dropped() {
        let re = compile("x*").unwrap();
        assert_eq!(char_spans("aaa", &re), vec![]);
        assert_eq!(char_spans("", &re), vec![]);
    }
}
