use regex::Regex;

/// The active search: compiled pattern plus the direction it was started in.
#[derive(Debug, Clone)]
pub struct Search {
    pub re: Regex,
    pub backwards: bool,
}

pub fn compile(pattern: &str) -> Result<Regex, regex::Error> {
    Regex::new(pattern)
}

/// Find the first matching line at or after/before `start` (inclusive).
pub fn find(lines: &[String], start: usize, backwards: bool, re: &Regex) -> Option<usize> {
    if backwards {
        (0..=start.min(lines.len().saturating_sub(1)))
            .rev()
            .find(|&i| re.is_match(&lines[i]))
    } else {
        (start..lines.len()).find(|&i| re.is_match(&lines[i]))
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
    fn empty_matches_are_dropped() {
        let re = compile("x*").unwrap();
        assert_eq!(char_spans("aaa", &re), vec![]);
        assert_eq!(char_spans("", &re), vec![]);
    }
}
