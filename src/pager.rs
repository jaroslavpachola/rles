use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyEventKind},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType},
};

use crate::commands::{Command, map_key};
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

fn event_loop(out: &mut impl Write, name: &str, lines: &[String]) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    // Last row is the prompt line.
    let mut view = View::new(rows.saturating_sub(1).max(1) as usize, cols as usize);
    let mut show_name = true;
    let total = lines.len();

    loop {
        draw(out, name, lines, &view, show_name)?;
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                show_name = false;
                match map_key(key) {
                    Command::Quit => return Ok(()),
                    Command::LineDown => view.scroll(1, total),
                    Command::LineUp => view.scroll(-1, total),
                    Command::PageDown => view.page_down(total),
                    Command::PageUp => view.page_up(total),
                    Command::HalfDown => view.half_down(total),
                    Command::HalfUp => view.half_up(total),
                    Command::GoTop => view.go_top(),
                    Command::GoBottom => view.go_bottom(total),
                    Command::Repaint | Command::None => {}
                }
            }
            Event::Resize(c, r) => {
                view.resize(r.saturating_sub(1).max(1) as usize, c as usize, total);
            }
            _ => {}
        }
    }
}

fn draw(
    out: &mut impl Write,
    name: &str,
    lines: &[String],
    view: &View,
    show_name: bool,
) -> io::Result<()> {
    for row in 0..view.height {
        queue!(
            out,
            cursor::MoveTo(0, row as u16),
            Clear(ClearType::CurrentLine)
        )?;
        match lines.get(view.top + row) {
            Some(line) => queue!(out, Print(clip(line, view.width)))?,
            None => queue!(out, Print("~"))?,
        }
    }
    queue!(
        out,
        cursor::MoveTo(0, view.height as u16),
        Clear(ClearType::CurrentLine)
    )?;
    if view.at_end(lines.len()) {
        queue!(
            out,
            SetAttribute(Attribute::Reverse),
            Print("(END)"),
            SetAttribute(Attribute::Reset)
        )?;
    } else if show_name {
        queue!(
            out,
            SetAttribute(Attribute::Reverse),
            Print(clip(name, view.width)),
            SetAttribute(Attribute::Reset)
        )?;
    } else {
        queue!(out, Print(":"))?;
    }
    out.flush()
}

/// Truncate a line to the first `width` characters (long lines are chopped,
/// horizontal scrolling arrives in a later release).
fn clip(line: &str, width: usize) -> String {
    line.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::clip;

    #[test]
    fn clip_truncates_by_chars() {
        assert_eq!(clip("hello", 10), "hello");
        assert_eq!(clip("hello", 3), "hel");
        assert_eq!(clip("žluťoučký", 4), "žluť");
        assert_eq!(clip("", 5), "");
    }
}
