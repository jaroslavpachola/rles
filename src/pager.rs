use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType},
};

use crate::commands::{Command, map_key};
use crate::search::{self, Search};
use crate::source::Source;
use crate::view::{HSCROLL_STEP, View};

/// Width of the line-number gutter, matching `less -N`.
const GUTTER: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub line_numbers: bool,
}

pub fn run(sources: Vec<Source>, opts: Options) -> io::Result<()> {
    let mut out = io::stdout().lock();
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
    let result = event_loop(&mut out, sources, opts);
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}

struct App {
    sources: Vec<Source>,
    current: usize,
    view: View,
    search: Option<Search>,
    message: Option<String>,
    show_name: bool,
    line_numbers: bool,
    /// Digits of a pending numeric count prefix.
    pending: String,
    /// Named positions: mark letter -> (source index, top line).
    marks: HashMap<char, (usize, usize)>,
}

fn event_loop(out: &mut impl Write, sources: Vec<Source>, opts: Options) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let mut app = App {
        sources,
        current: 0,
        // Last row is the prompt line.
        view: View::new(rows.saturating_sub(1).max(1) as usize, cols as usize),
        search: None,
        message: None,
        show_name: true,
        line_numbers: opts.line_numbers,
        pending: String::new(),
        marks: HashMap::new(),
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
                let total = app.lines().len();
                app.view
                    .resize(r.saturating_sub(1).max(1) as usize, c as usize, total);
            }
            _ => {}
        }
    }
}

impl App {
    fn lines(&self) -> &[String] {
        &self.sources[self.current].lines
    }

    fn name(&self) -> &str {
        &self.sources[self.current].name
    }

    /// Remember the current position as the `'` mark before a jump.
    fn remember_position(&mut self) {
        self.marks.insert('\'', (self.current, self.view.top));
    }

    /// Returns false when the pager should quit.
    fn handle_key(&mut self, out: &mut impl Write, cmd: Command) -> io::Result<bool> {
        if let Command::Digit(d) = cmd {
            if self.pending.len() < 9 {
                self.pending.push(char::from_digit(d, 10).unwrap());
            }
            return Ok(true);
        }
        let count: Option<usize> = self.pending.parse().ok();
        self.pending.clear();

        if matches!(
            cmd,
            Command::GoTop
                | Command::GoBottom
                | Command::GoPercent
                | Command::SearchForward
                | Command::SearchBackward
                | Command::NextMatch
                | Command::PrevMatch
        ) {
            self.remember_position();
        }

        let total = self.lines().len();
        match cmd {
            Command::Quit => return Ok(false),
            Command::LineDown => self.view.scroll(count.unwrap_or(1) as isize, total),
            Command::LineUp => self.view.scroll(-(count.unwrap_or(1) as isize), total),
            Command::PageDown => self.view.page_down(total),
            Command::PageUp => self.view.page_up(total),
            Command::HalfDown => self.view.half_down(total),
            Command::HalfUp => self.view.half_up(total),
            Command::GoTop => match count {
                Some(n) => self.view.go_line(n, total),
                None => self.view.go_top(),
            },
            Command::GoBottom => match count {
                Some(n) => self.view.go_line(n, total),
                None => self.view.go_bottom(total),
            },
            Command::GoPercent => self.view.go_percent(count.unwrap_or(0), total),
            Command::ScrollLeft => self.view.scroll_left(count.unwrap_or(HSCROLL_STEP)),
            Command::ScrollRight => self.view.scroll_right(count.unwrap_or(HSCROLL_STEP)),
            Command::SearchForward => self.prompt_search(out, false)?,
            Command::SearchBackward => self.prompt_search(out, true)?,
            Command::NextMatch => self.repeat_search(false, count.unwrap_or(1)),
            Command::PrevMatch => self.repeat_search(true, count.unwrap_or(1)),
            Command::OptionPrompt => self.prompt_option(out)?,
            Command::ColonPrompt => return self.prompt_colon(out),
            Command::FileInfo => self.file_info(),
            Command::Follow => self.follow(out)?,
            Command::MarkSet => self.mark_set(out)?,
            Command::MarkGoto => self.mark_goto(out)?,
            Command::Digit(_) => unreachable!(),
            Command::Repaint | Command::None => {}
        }
        Ok(true)
    }

    fn file_info(&mut self) {
        let total = self.lines().len();
        let last = (self.view.top + self.view.height).min(total);
        self.message = Some(format!(
            "{} lines {}-{}/{} (file {} of {})",
            self.name(),
            (self.view.top + 1).min(total),
            last,
            total,
            self.current + 1,
            self.sources.len()
        ));
    }

    /// `:` prompt — n(ext file), p(revious file), q(uit).
    fn prompt_colon(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let Some(key) = self.prompt_char(out, ":")? else {
            return Ok(true);
        };
        match key {
            'n' => self.switch_file(1),
            'p' => self.switch_file(-1),
            'q' => return Ok(false),
            c => self.message = Some(format!("Unknown command: :{c}")),
        }
        Ok(true)
    }

    fn switch_file(&mut self, delta: isize) {
        let target = self.current as isize + delta;
        if target < 0 || target >= self.sources.len() as isize {
            self.message = Some(
                if delta > 0 {
                    "No next file"
                } else {
                    "No previous file"
                }
                .into(),
            );
            return;
        }
        self.sources[self.current].saved = (self.view.top, self.view.left);
        self.current = target as usize;
        let (top, left) = self.sources[self.current].saved;
        let total = self.lines().len();
        self.view.top = top.min(self.view.max_top(total));
        self.view.left = left;
        self.file_info();
    }

    fn mark_set(&mut self, out: &mut impl Write) -> io::Result<()> {
        let Some(c) = self.prompt_char(out, "mark: ")? else {
            return Ok(());
        };
        if c.is_ascii_alphabetic() {
            self.marks.insert(c, (self.current, self.view.top));
        } else {
            self.message = Some("Marks must be letters".into());
        }
        Ok(())
    }

    fn mark_goto(&mut self, out: &mut impl Write) -> io::Result<()> {
        let Some(c) = self.prompt_char(out, "goto mark: ")? else {
            return Ok(());
        };
        match self.marks.get(&c).copied() {
            Some((source, top)) => {
                self.remember_position();
                if source != self.current {
                    self.sources[self.current].saved = (self.view.top, self.view.left);
                    self.current = source;
                }
                let total = self.lines().len();
                self.view.top = top.min(self.view.max_top(total));
            }
            None => self.message = Some(format!("No mark: {c}")),
        }
        Ok(())
    }

    /// Follow the current file like `tail -f` until a key is pressed.
    fn follow(&mut self, out: &mut impl Write) -> io::Result<()> {
        if self.sources[self.current].path.is_none() {
            self.message = Some("Cannot follow standard input".into());
            return Ok(());
        }
        loop {
            let total = self.lines().len();
            self.view.go_bottom(total);
            self.draw_with_prompt(out, Some("Waiting for data... (press any key to stop)"))?;
            if event::poll(Duration::from_millis(300))? {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => return Ok(()),
                    Event::Resize(c, r) => {
                        self.view.resize(
                            r.saturating_sub(1).max(1) as usize,
                            c as usize,
                            self.lines().len(),
                        );
                    }
                    _ => {}
                }
            } else if let Err(err) = self.sources[self.current].reload() {
                self.message = Some(format!("Cannot follow: {err}"));
                return Ok(());
            }
        }
    }

    /// Show a one-line prompt and read a single character; None on cancel.
    fn prompt_char(&mut self, out: &mut impl Write, prompt: &str) -> io::Result<Option<char>> {
        queue!(
            out,
            cursor::MoveTo(0, self.view.height as u16),
            Clear(ClearType::CurrentLine),
            Print(prompt)
        )?;
        out.flush()?;
        loop {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(None);
            }
            match key.code {
                KeyCode::Char(c) => return Ok(Some(c)),
                KeyCode::Esc => return Ok(None),
                _ => return Ok(None),
            }
        }
    }

    fn prompt_option(&mut self, out: &mut impl Write) -> io::Result<()> {
        let Some(c) = self.prompt_char(out, "-")? else {
            return Ok(());
        };
        match c {
            'N' => {
                self.line_numbers = !self.line_numbers;
                self.message = Some(
                    if self.line_numbers {
                        "Line numbers on"
                    } else {
                        "Line numbers off"
                    }
                    .into(),
                );
            }
            c => self.message = Some(format!("Unknown option: -{c}")),
        }
        Ok(())
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

    /// Columns available for line content (the gutter takes its share).
    fn content_width(&self) -> usize {
        if self.line_numbers {
            self.view.width.saturating_sub(GUTTER)
        } else {
            self.view.width
        }
    }

    fn repeat_search(&mut self, reverse: bool, times: usize) {
        match &self.search {
            Some(s) => {
                let backwards = s.backwards ^ reverse;
                for _ in 0..times.max(1) {
                    self.run_search(backwards);
                    if self.message.is_some() {
                        break;
                    }
                }
            }
            None => self.message = Some("No previous search".into()),
        }
    }

    fn run_search(&mut self, backwards: bool) {
        let Some(s) = &self.search else { return };
        let top = self.view.top;
        let lines = self.lines();
        let found = if backwards {
            top.checked_sub(1)
                .and_then(|start| search::find(lines, start, true, &s.re))
        } else {
            search::find(lines, top + 1, false, &s.re)
        };
        match found {
            Some(line) => {
                self.view.top = line.min(self.view.max_top(self.lines().len()));
            }
            None => self.message = Some("Pattern not found".into()),
        }
    }

    fn draw(&self, out: &mut impl Write) -> io::Result<()> {
        self.draw_with_prompt(out, None)
    }

    fn draw_with_prompt(&self, out: &mut impl Write, prompt: Option<&str>) -> io::Result<()> {
        for row in 0..self.view.height {
            queue!(
                out,
                cursor::MoveTo(0, row as u16),
                Clear(ClearType::CurrentLine)
            )?;
            match self.lines().get(self.view.top + row) {
                Some(line) => {
                    if self.line_numbers {
                        queue!(out, Print(format!("{:>7} ", self.view.top + row + 1)))?;
                    }
                    self.draw_content_line(out, line)?;
                }
                None => queue!(out, Print("~"))?,
            }
        }
        queue!(
            out,
            cursor::MoveTo(0, self.view.height as u16),
            Clear(ClearType::CurrentLine)
        )?;
        if let Some(text) = prompt {
            draw_reverse(out, &clip(text, 0, self.view.width))?;
        } else if let Some(msg) = &self.message {
            draw_reverse(out, &clip(msg, 0, self.view.width))?;
        } else if !self.pending.is_empty() {
            queue!(out, Print(format!(":{}", self.pending)))?;
        } else if self.view.at_end(self.lines().len()) {
            draw_reverse(out, "(END)")?;
        } else if self.show_name {
            draw_reverse(out, &clip(self.name(), 0, self.view.width))?;
        } else {
            queue!(out, Print(":"))?;
        }
        out.flush()
    }

    fn draw_content_line(&self, out: &mut impl Write, line: &str) -> io::Result<()> {
        let (left, width) = (self.view.left, self.content_width());
        let spans = match &self.search {
            Some(s) => search::char_spans(line, &s.re),
            None => Vec::new(),
        };
        if spans.is_empty() {
            return queue!(out, Print(clip(line, left, width)));
        }
        for (text, highlighted) in segments(line, left, width, &spans) {
            if highlighted {
                draw_reverse(out, &text)?;
            } else {
                queue!(out, Print(text))?;
            }
        }
        Ok(())
    }
}

/// Split the visible window (chars `left..left+width`) of `line` into runs of
/// (text, highlighted), where `spans` are sorted, non-overlapping
/// (start, end) char ranges over the whole line.
fn segments(
    line: &str,
    left: usize,
    width: usize,
    spans: &[(usize, usize)],
) -> Vec<(String, bool)> {
    let mut result: Vec<(String, bool)> = Vec::new();
    for (idx, ch) in line.chars().enumerate().skip(left).take(width) {
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

/// The visible window of a line: chars `left..left+width` (chopped, not wrapped).
fn clip(line: &str, left: usize, width: usize) -> String {
    line.chars().skip(left).take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::{clip, segments};

    #[test]
    fn clip_truncates_by_chars() {
        assert_eq!(clip("hello", 0, 10), "hello");
        assert_eq!(clip("hello", 0, 3), "hel");
        assert_eq!(clip("žluťoučký", 0, 4), "žluť");
        assert_eq!(clip("", 0, 5), "");
    }

    #[test]
    fn clip_honours_left_offset() {
        assert_eq!(clip("abcdef", 2, 3), "cde");
        assert_eq!(clip("abcdef", 10, 3), "");
    }

    #[test]
    fn segments_split_on_span_borders() {
        let spans = vec![(2, 4)];
        assert_eq!(
            segments("abcdef", 0, 80, &spans),
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
            segments("abcdef", 0, 3, &spans),
            vec![("ab".to_string(), false), ("c".to_string(), true)]
        );
    }

    #[test]
    fn segments_shifted_by_left_offset() {
        let spans = vec![(2, 4)];
        assert_eq!(
            segments("abcdef", 3, 80, &spans),
            vec![("d".to_string(), true), ("ef".to_string(), false)]
        );
    }

    #[test]
    fn segments_whole_line_highlighted() {
        let spans = vec![(0, 3)];
        assert_eq!(
            segments("abc", 0, 80, &spans),
            vec![("abc".to_string(), true)]
        );
    }
}
