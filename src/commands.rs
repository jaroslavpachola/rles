use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A pager action, decoupled from the raw key event so the mapping is testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Quit,
    LineDown,
    LineUp,
    PageDown,
    PageUp,
    HalfDown,
    HalfUp,
    GoTop,
    GoBottom,
    SearchForward,
    SearchBackward,
    NextMatch,
    PrevMatch,
    ScrollLeft,
    ScrollRight,
    GoPercent,
    /// A digit of a numeric count prefix (`12j` scrolls 12 lines).
    Digit(u32),
    /// `-` followed by an option letter toggles that option at runtime.
    OptionPrompt,
    /// `:` followed by n/p/q — file switching, quit.
    ColonPrompt,
    FileInfo,
    Follow,
    MarkSet,
    MarkGoto,
    Help,
    Repaint,
    None,
}

pub fn map_key(key: KeyEvent) -> Command {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Command::Quit,
            KeyCode::Char('f') | KeyCode::Char('v') => Command::PageDown,
            KeyCode::Char('b') => Command::PageUp,
            KeyCode::Char('d') => Command::HalfDown,
            KeyCode::Char('u') => Command::HalfUp,
            KeyCode::Char('e') | KeyCode::Char('n') => Command::LineDown,
            KeyCode::Char('y') | KeyCode::Char('p') | KeyCode::Char('k') => Command::LineUp,
            KeyCode::Char('l') => Command::Repaint,
            KeyCode::Char('g') => Command::FileInfo,
            _ => Command::None,
        };
    }
    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => Command::Digit(c.to_digit(10).unwrap()),
        KeyCode::Char('q') | KeyCode::Char('Q') => Command::Quit,
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => Command::LineDown,
        KeyCode::Char('k') | KeyCode::Up => Command::LineUp,
        KeyCode::Char(' ') | KeyCode::Char('f') | KeyCode::PageDown => Command::PageDown,
        KeyCode::Char('b') | KeyCode::PageUp => Command::PageUp,
        KeyCode::Char('d') => Command::HalfDown,
        KeyCode::Char('u') => Command::HalfUp,
        KeyCode::Char('g') | KeyCode::Char('<') | KeyCode::Home => Command::GoTop,
        KeyCode::Char('G') | KeyCode::Char('>') | KeyCode::End => Command::GoBottom,
        KeyCode::Left => Command::ScrollLeft,
        KeyCode::Right => Command::ScrollRight,
        KeyCode::Char('p') | KeyCode::Char('%') => Command::GoPercent,
        KeyCode::Char('-') => Command::OptionPrompt,
        KeyCode::Char(':') => Command::ColonPrompt,
        KeyCode::Char('=') => Command::FileInfo,
        KeyCode::Char('F') => Command::Follow,
        KeyCode::Char('m') => Command::MarkSet,
        KeyCode::Char('\'') => Command::MarkGoto,
        KeyCode::Char('/') => Command::SearchForward,
        KeyCode::Char('?') => Command::SearchBackward,
        KeyCode::Char('n') => Command::NextMatch,
        KeyCode::Char('N') => Command::PrevMatch,
        KeyCode::Char('h') | KeyCode::Char('H') => Command::Help,
        KeyCode::Char('r') => Command::Repaint,
        _ => Command::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn basic_movement_keys() {
        assert_eq!(map_key(key(KeyCode::Char('q'))), Command::Quit);
        assert_eq!(map_key(key(KeyCode::Char('j'))), Command::LineDown);
        assert_eq!(map_key(key(KeyCode::Down)), Command::LineDown);
        assert_eq!(map_key(key(KeyCode::Char('k'))), Command::LineUp);
        assert_eq!(map_key(key(KeyCode::Char(' '))), Command::PageDown);
        assert_eq!(map_key(key(KeyCode::Char('b'))), Command::PageUp);
        assert_eq!(map_key(key(KeyCode::Char('g'))), Command::GoTop);
        assert_eq!(map_key(key(KeyCode::Char('G'))), Command::GoBottom);
    }

    #[test]
    fn control_keys() {
        assert_eq!(map_key(ctrl('c')), Command::Quit);
        assert_eq!(map_key(ctrl('f')), Command::PageDown);
        assert_eq!(map_key(ctrl('b')), Command::PageUp);
        assert_eq!(map_key(ctrl('d')), Command::HalfDown);
        assert_eq!(map_key(ctrl('u')), Command::HalfUp);
    }

    #[test]
    fn search_keys() {
        assert_eq!(map_key(key(KeyCode::Char('/'))), Command::SearchForward);
        assert_eq!(map_key(key(KeyCode::Char('?'))), Command::SearchBackward);
        assert_eq!(map_key(key(KeyCode::Char('n'))), Command::NextMatch);
        assert_eq!(map_key(key(KeyCode::Char('N'))), Command::PrevMatch);
    }

    #[test]
    fn navigation_extras() {
        assert_eq!(map_key(key(KeyCode::Char('4'))), Command::Digit(4));
        assert_eq!(map_key(key(KeyCode::Char('0'))), Command::Digit(0));
        assert_eq!(map_key(key(KeyCode::Left)), Command::ScrollLeft);
        assert_eq!(map_key(key(KeyCode::Right)), Command::ScrollRight);
        assert_eq!(map_key(key(KeyCode::Char('p'))), Command::GoPercent);
        assert_eq!(map_key(key(KeyCode::Char('%'))), Command::GoPercent);
        assert_eq!(map_key(key(KeyCode::Char('-'))), Command::OptionPrompt);
    }

    #[test]
    fn file_and_mark_keys() {
        assert_eq!(map_key(key(KeyCode::Char(':'))), Command::ColonPrompt);
        assert_eq!(map_key(key(KeyCode::Char('='))), Command::FileInfo);
        assert_eq!(map_key(ctrl('g')), Command::FileInfo);
        assert_eq!(map_key(key(KeyCode::Char('F'))), Command::Follow);
        assert_eq!(map_key(key(KeyCode::Char('m'))), Command::MarkSet);
        assert_eq!(map_key(key(KeyCode::Char('\''))), Command::MarkGoto);
        assert_eq!(map_key(key(KeyCode::Char('h'))), Command::Help);
    }

    #[test]
    fn unknown_keys_do_nothing() {
        assert_eq!(map_key(key(KeyCode::Char('x'))), Command::None);
        assert_eq!(map_key(key(KeyCode::Tab)), Command::None);
    }
}
