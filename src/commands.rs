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
            _ => Command::None,
        };
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Command::Quit,
        KeyCode::Char('j') | KeyCode::Down | KeyCode::Enter => Command::LineDown,
        KeyCode::Char('k') | KeyCode::Up => Command::LineUp,
        KeyCode::Char(' ') | KeyCode::Char('f') | KeyCode::PageDown => Command::PageDown,
        KeyCode::Char('b') | KeyCode::PageUp => Command::PageUp,
        KeyCode::Char('d') => Command::HalfDown,
        KeyCode::Char('u') => Command::HalfUp,
        KeyCode::Char('g') | KeyCode::Char('<') | KeyCode::Home => Command::GoTop,
        KeyCode::Char('G') | KeyCode::Char('>') | KeyCode::End => Command::GoBottom,
        KeyCode::Char('/') => Command::SearchForward,
        KeyCode::Char('?') => Command::SearchBackward,
        KeyCode::Char('n') => Command::NextMatch,
        KeyCode::Char('N') => Command::PrevMatch,
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
    fn unknown_keys_do_nothing() {
        assert_eq!(map_key(key(KeyCode::Char('x'))), Command::None);
        assert_eq!(map_key(key(KeyCode::Tab)), Command::None);
    }
}
