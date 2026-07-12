use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType},
};

use crate::commands::{Command, map_key};
use crate::search::{self, Search};
use crate::view::View;

pub fn run(name: &str, lines: &[String]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
    let result = event_loop(&mut out, name, lines);
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}

struct App<'a> {
    name: &'a str,
    lines: &'a [String],
    view: View,
    search: Option<Search>,
    message: Option<String>,
    show_name: bool,
}

fn event_loop(out: &mut impl Write, name: &str, lines: &[String]) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let mut app = App {
        name,
        lines,
        // Last row is the prompt line.
        view: View::new(rows.saturating_sub(1).max(1) as usize, cols as usize),
        search: None,
        message: None,
        show_name: true,
    };

    loop {
        app.draw(out)?;
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                app.show_name = false;
                app.message = None;
                if !app.handle_key(out, map_key(key))? {
                    return Ok(());
                }
            }
            Event::Resize(c, r) => {
                let total = app.lines.len();
                app.view
                    .resize(r.saturating_sub(1).max(1) as usize, c as usize, total);
            }
            _ => {}
        }
    }
}

impl App<'_> {
    /// Returns false when the pager should quit.
    fn handle_key(&mut self, out: &mut impl Write, cmd: Command) -> io::Result<bool> {
        let total = self.lines.len();
        match cmd {
            Command::Quit => return Ok(false),
            Command::LineDown => self.view.scroll(1, total),
            Command::LineUp => self.view.scroll(-1, total),
            Command::PageDown => self.view.page_down(total),
            Command::PageUp => self.view.page_up(total),
            Command::HalfDown => self.view.half_down(total),
            Command::HalfUp => self.view.half_up(total),
            Command::GoTop => self.view.go_top(),
            Command::GoBottom => self.view.go_bottom(total),
            Command::SearchForward => self.prompt_search(out, false)?,
            Command::SearchBackward => self.prompt_search(out, true)?,
            Command::NextMatch => self.repeat_search(false),
            Command::PrevMatch => self.repeat_search(true),
            Command::Repaint | Command::None => {}
        }
        Ok(true)
    }

    fn prompt_search(&mut self, out: &mut impl Write, backwards: bool) -> io::Result<()> {
        let prompt = if backwards { '?' } else { '/' };
        let Some(pattern) = read_line(out, self.view.height, prompt)? else {
            return Ok(());
        };
        if pattern.is_empty() {
            // Bare Enter repeats the last search in the given direction.
            match self.search.take() {
                Some(prev) => self.search = Some(Search { backwards, ..prev }),
                None => return Ok(()),
            }
        } else {
            match search::compile(&pattern) {
                Ok(re) => self.search = Some(Search { re, backwards }),
                Err(_) => {
                    self.message = Some(format!("Invalid pattern: {pattern}"));
                    return Ok(());
                }
            }
        }
        self.run_search(backwards);
        Ok(())
    }

    fn repeat_search(&mut self, reverse: bool) {
        match &self.search {
            Some(s) => self.run_search(s.backwards ^ reverse),
            None => self.message = Some("No previous search".into()),
        }
    }

    fn run_search(&mut self, backwards: bool) {
        let Some(s) = &self.search else { return };
        let top = self.view.top;
        let found = if backwards {
            top.checked_sub(1)
                .and_then(|start| search::find(self.lines, start, true, &s.re))
        } else {
            search::find(self.lines, top + 1, false, &s.re)
        };
        match found {
            Some(line) => {
                self.view.top = line.min(self.view.max_top(self.lines.len()));
            }
            None => self.message = Some("Pattern not found".into()),
        }
    }

    fn draw(&self, out: &mut impl Write) -> io::Result<()> {
        for row in 0..self.view.height {
            queue!(
                out,
                cursor::MoveTo(0, row as u16),
                Clear(ClearType::CurrentLine)
            )?;
            match self.lines.get(self.view.top + row) {
                Some(line) => self.draw_content_line(out, line)?,
                None => queue!(out, Print("~"))?,
            }
        }
        queue!(
            out,
            cursor::MoveTo(0, self.view.height as u16),
            Clear(ClearType::CurrentLine)
        )?;
        if let Some(msg) = &self.message {
            draw_reverse(out, &clip(msg, self.view.width))?;
        } else if self.view.at_end(self.lines.len()) {
            draw_reverse(out, "(END)")?;
        } else if self.show_name {
            draw_reverse(out, &clip(self.name, self.view.width))?;
        } else {
            queue!(out, Print(":"))?;
        }
        out.flush()
    }

    fn draw_content_line(&self, out: &mut impl Write, line: &str) -> io::Result<()> {
        let spans = match &self.search {
            Some(s) => search::char_spans(line, &s.re),
            None => Vec::new(),
        };
        if spans.is_empty() {
            return queue!(out, Print(clip(line, self.view.width)));
        }
        for (text, highlighted) in segments(line, self.view.width, &spans) {
            if highlighted {
                draw_reverse(out, &text)?;
            } else {
                queue!(out, Print(text))?;
            }
        }
        Ok(())
    }
}

/// Split the first `width` chars of `line` into runs of (text, highlighted),
/// where `spans` are sorted, non-overlapping (start, end) char ranges.
fn segments(line: &str, width: usize, spans: &[(usize, usize)]) -> Vec<(String, bool)> {
    let mut result: Vec<(String, bool)> = Vec::new();
    for (idx, ch) in line.chars().take(width).enumerate() {
        let highlighted = spans.iter().any(|&(s, e)| idx >= s && idx < e);
        match result.last_mut() {
            Some((text, h)) if *h == highlighted => text.push(ch),
            _ => result.push((ch.to_string(), highlighted)),
        }
    }
    result
}

fn draw_reverse(out: &mut impl Write, text: &str) -> io::Result<()> {
    queue!(
        out,
        SetAttribute(Attribute::Reverse),
        Print(text),
        SetAttribute(Attribute::Reset)
    )
}

/// Read a line of input on the prompt row. Returns None when cancelled
/// (Esc, ctrl-c, ctrl-g, or backspace on empty input, like less).
fn read_line(out: &mut impl Write, row: usize, prompt: char) -> io::Result<Option<String>> {
    let mut input = String::new();
    loop {
        queue!(
            out,
            cursor::MoveTo(0, row as u16),
            Clear(ClearType::CurrentLine),
            Print(prompt),
            Print(&input)
        )?;
        out.flush()?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('g') => return Ok(None),
                _ => continue,
            }
        }
        match key.code {
            KeyCode::Enter => return Ok(Some(input)),
            KeyCode::Esc => return Ok(None),
            KeyCode::Backspace => {
                if input.pop().is_none() {
                    return Ok(None);
                }
            }
            KeyCode::Char(c) => input.push(c),
            _ => {}
        }
    }
}

/// Truncate a line to the first `width` characters (long lines are chopped,
/// horizontal scrolling arrives in a later release).
fn clip(line: &str, width: usize) -> String {
    line.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::{clip, segments};

    #[test]
    fn clip_truncates_by_chars() {
        assert_eq!(clip("hello", 10), "hello");
        assert_eq!(clip("hello", 3), "hel");
        assert_eq!(clip("žluťoučký", 4), "žluť");
        assert_eq!(clip("", 5), "");
    }

    #[test]
    fn segments_split_on_span_borders() {
        let spans = vec![(2, 4)];
        assert_eq!(
            segments("abcdef", 80, &spans),
            vec![
                ("ab".to_string(), false),
                ("cd".to_string(), true),
                ("ef".to_string(), false)
            ]
        );
    }

    #[test]
    fn segments_respect_width() {
        let spans = vec![(2, 4)];
        assert_eq!(
            segments("abcdef", 3, &spans),
            vec![("ab".to_string(), false), ("c".to_string(), true)]
        );
    }

    #[test]
    fn segments_whole_line_highlighted() {
        let spans = vec![(0, 3)];
        assert_eq!(segments("abc", 80, &spans), vec![("abc".to_string(), true)]);
    }
}
