use std::io::{self, Read};
use std::path::PathBuf;

/// One viewed input: a file on disk or the drained standard input.
pub struct Source {
    pub name: String,
    /// None for stdin — such a source cannot be reloaded or followed.
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    /// Saved (top, left) viewport, restored when switching back to this file.
    pub saved: (usize, usize),
}

impl Source {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let bytes =
            std::fs::read(path).map_err(|e| io::Error::new(e.kind(), format!("{path}: {e}")))?;
        Ok(Self {
            name: path.to_owned(),
            path: Some(PathBuf::from(path)),
            lines: split_lines(&bytes),
            saved: (0, 0),
        })
    }

    pub fn from_stdin() -> io::Result<Self> {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf)?;
        Ok(Self {
            name: "(stdin)".to_owned(),
            path: None,
            lines: split_lines(&buf),
            saved: (0, 0),
        })
    }

    /// Re-read the backing file (used by follow mode).
    pub fn reload(&mut self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let bytes = std::fs::read(path)?;
        self.lines = split_lines(&bytes);
        Ok(())
    }
}

fn split_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_handles_trailing_newline_and_crlf() {
        assert_eq!(split_lines(b"a\nb\n"), vec!["a", "b"]);
        assert_eq!(split_lines(b"a\nb"), vec!["a", "b"]);
        assert_eq!(split_lines(b"a\r\nb\r\n"), vec!["a", "b"]);
        assert!(split_lines(b"").is_empty());
    }

    #[test]
    fn reload_picks_up_appended_lines() {
        let path = std::env::temp_dir().join(format!("rles-test-{}", std::process::id()));
        std::fs::write(&path, "one\n").unwrap();
        let mut src = Source::from_file(path.to_str().unwrap()).unwrap();
        assert_eq!(src.lines, vec!["one"]);
        std::fs::write(&path, "one\ntwo\n").unwrap();
        src.reload().unwrap();
        assert_eq!(src.lines, vec!["one", "two"]);
        std::fs::remove_file(&path).unwrap();
    }
}
