//! ANSI-aware line handling: colored input (git diff, grep --color, …) is
//! rendered with its SGR styling intact, while widths, horizontal scrolling
//! and search operate on the visible text only. Non-SGR escape sequences are
//! dropped so cursor-movement garbage cannot corrupt the screen.

enum Token {
    Text(char),
    /// A complete SGR sequence, e.g. `\x1b[1;31m`.
    Sgr(String),
}

fn tokenize(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            tokens.push(Token::Text(c));
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                let mut body = String::new();
                let mut terminator = None;
                for c in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&c) {
                        terminator = Some(c);
                        break;
                    }
                    body.push(c);
                }
                if terminator == Some('m') {
                    tokens.push(Token::Sgr(format!("\x1b[{body}m")));
                }
                // Other CSI sequences are dropped.
            }
            Some('(') | Some(')') => {
                // Charset designation: ESC ( X — drop introducer and charset.
                chars.next();
                chars.next();
            }
            Some(']') => {
                // OSC: runs until BEL or ST (ESC \).
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '\x07' {
                        break;
                    }
                    if c == '\x1b' {
                        chars.next();
                        break;
                    }
                }
            }
            Some(_) => {
                // Other two-char escape (ESC 7, ESC =, …): drop the char.
                chars.next();
            }
            None => {}
        }
    }
    tokens
}

/// The visible text with all escape sequences removed.
pub fn strip(line: &str) -> std::borrow::Cow<'_, str> {
    if !line.contains('\x1b') {
        return std::borrow::Cow::Borrowed(line);
    }
    tokenize(line)
        .into_iter()
        .filter_map(|t| match t {
            Token::Text(c) => Some(c),
            Token::Sgr(_) => None,
        })
        .collect::<String>()
        .into()
}

/// Render the window of visible chars `left..left+width`, keeping SGR
/// styling and reverse-highlighting `spans` (char ranges over the stripped
/// text). Always ends with a full attribute reset.
pub fn render_window(line: &str, left: usize, width: usize, spans: &[(usize, usize)]) -> String {
    let mut result = String::new();
    let mut idx = 0usize; // index into the stripped text
    let mut highlighted = false;
    for token in tokenize(line) {
        match token {
            Token::Sgr(seq) => {
                if idx >= left + width {
                    break;
                }
                result.push_str(&seq);
                if highlighted {
                    // An SGR may have cleared the reverse attribute; re-assert.
                    result.push_str("\x1b[7m");
                }
            }
            Token::Text(c) => {
                if idx >= left && idx < left + width {
                    let in_span = spans.iter().any(|&(s, e)| idx >= s && idx < e);
                    if in_span != highlighted {
                        result.push_str(if in_span { "\x1b[7m" } else { "\x1b[27m" });
                        highlighted = in_span;
                    }
                    result.push(c);
                }
                idx += 1;
            }
        }
    }
    result.push_str("\x1b[0m");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_sgr_and_other_escapes() {
        assert_eq!(strip("plain"), "plain");
        assert_eq!(strip("\x1b[1;31mred\x1b[0m rest"), "red rest");
        assert_eq!(strip("a\x1b[2Kb"), "ab"); // non-SGR CSI dropped
        assert_eq!(strip("a\x1b(Bb"), "ab"); // charset escape dropped
        assert_eq!(strip("a\x1b]0;title\x07b"), "ab"); // OSC dropped
    }

    #[test]
    fn strip_borrows_plain_lines() {
        assert!(matches!(
            strip("no escapes here"),
            std::borrow::Cow::Borrowed(_)
        ));
    }

    #[test]
    fn render_keeps_sgr_and_resets() {
        assert_eq!(
            render_window("\x1b[31mred\x1b[0m", 0, 80, &[]),
            "\x1b[31mred\x1b[0m\x1b[0m"
        );
    }

    #[test]
    fn render_window_counts_visible_chars_only() {
        // Window over chars 1..3 of "abcd"; color codes do not consume width.
        // The trailing SGR falls outside the window; only the final reset remains.
        assert_eq!(
            render_window("a\x1b[32mbcd\x1b[0m", 1, 2, &[]),
            "\x1b[32mbc\x1b[0m"
        );
    }

    #[test]
    fn render_highlights_span_transitions() {
        assert_eq!(
            render_window("abcd", 0, 80, &[(1, 3)]),
            "a\x1b[7mbc\x1b[27md\x1b[0m"
        );
    }

    #[test]
    fn render_reasserts_highlight_after_sgr() {
        assert_eq!(
            render_window("a\x1b[31mbc", 0, 80, &[(0, 3)]),
            "\x1b[7ma\x1b[31m\x1b[7mbc\x1b[0m"
        );
    }

    #[test]
    fn non_sgr_csi_does_not_leak_into_output() {
        assert_eq!(render_window("a\x1b[2Kb", 0, 80, &[]), "ab\x1b[0m");
    }
}
