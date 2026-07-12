/// Viewport over a buffer of lines: which line is at the top of the screen
/// and how many rows/columns of content are visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    pub top: usize,
    pub left: usize,
    pub height: usize,
    pub width: usize,
}

/// Columns moved per horizontal scroll step.
pub const HSCROLL_STEP: usize = 8;

impl View {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            top: 0,
            left: 0,
            height,
            width,
        }
    }

    /// Highest allowed `top` so the last line stays on screen.
    pub fn max_top(&self, total: usize) -> usize {
        total.saturating_sub(self.height)
    }

    pub fn scroll(&mut self, delta: isize, total: usize) {
        let top = self.top as isize + delta;
        self.top = top.clamp(0, self.max_top(total) as isize) as usize;
    }

    pub fn page_down(&mut self, total: usize) {
        self.scroll(self.height as isize, total);
    }

    pub fn page_up(&mut self, total: usize) {
        self.scroll(-(self.height as isize), total);
    }

    pub fn half_down(&mut self, total: usize) {
        self.scroll((self.height / 2).max(1) as isize, total);
    }

    pub fn half_up(&mut self, total: usize) {
        self.scroll(-((self.height / 2).max(1) as isize), total);
    }

    pub fn go_top(&mut self) {
        self.top = 0;
    }

    /// Jump so the given 1-based line is at the top (clamped).
    pub fn go_line(&mut self, line: usize, total: usize) {
        self.top = line.saturating_sub(1).min(self.max_top(total));
    }

    /// Jump to a percentage of the buffer (0–100).
    pub fn go_percent(&mut self, percent: usize, total: usize) {
        self.top = (total * percent.min(100) / 100).min(self.max_top(total));
    }

    pub fn scroll_right(&mut self, cols: usize) {
        // Arbitrary cap so `left` cannot run away; lines longer than this
        // are unreachable but the viewport stays sane.
        self.left = (self.left + cols).min(100_000);
    }

    pub fn scroll_left(&mut self, cols: usize) {
        self.left = self.left.saturating_sub(cols);
    }

    pub fn go_bottom(&mut self, total: usize) {
        self.top = self.max_top(total);
    }

    pub fn at_end(&self, total: usize) -> bool {
        self.top >= self.max_top(total)
    }

    pub fn resize(&mut self, height: usize, width: usize, total: usize) {
        self.height = height;
        self.width = width;
        self.top = self.top.min(self.max_top(total));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> View {
        View::new(10, 80)
    }

    #[test]
    fn scroll_clamps_at_start() {
        let mut v = view();
        v.scroll(-5, 100);
        assert_eq!(v.top, 0);
    }

    #[test]
    fn scroll_clamps_at_end() {
        let mut v = view();
        v.scroll(1000, 100);
        assert_eq!(v.top, 90);
    }

    #[test]
    fn short_buffer_never_scrolls() {
        let mut v = view();
        v.page_down(5);
        assert_eq!(v.top, 0);
        assert!(v.at_end(5));
    }

    #[test]
    fn paging_moves_by_screenful() {
        let mut v = view();
        v.page_down(100);
        assert_eq!(v.top, 10);
        v.half_down(100);
        assert_eq!(v.top, 15);
        v.half_up(100);
        v.page_up(100);
        assert_eq!(v.top, 0);
    }

    #[test]
    fn bottom_and_end_detection() {
        let mut v = view();
        assert!(!v.at_end(100));
        v.go_bottom(100);
        assert_eq!(v.top, 90);
        assert!(v.at_end(100));
        v.go_top();
        assert_eq!(v.top, 0);
    }

    #[test]
    fn resize_reclamps_top() {
        let mut v = view();
        v.go_bottom(100);
        v.resize(50, 80, 100);
        assert_eq!(v.top, 50);
    }

    #[test]
    fn go_line_is_one_based_and_clamped() {
        let mut v = view();
        v.go_line(5, 100);
        assert_eq!(v.top, 4);
        v.go_line(0, 100);
        assert_eq!(v.top, 0);
        v.go_line(1000, 100);
        assert_eq!(v.top, 90);
    }

    #[test]
    fn go_percent_spans_the_buffer() {
        let mut v = view();
        v.go_percent(0, 100);
        assert_eq!(v.top, 0);
        v.go_percent(50, 100);
        assert_eq!(v.top, 50);
        v.go_percent(100, 100);
        assert_eq!(v.top, 90);
        v.go_percent(200, 100);
        assert_eq!(v.top, 90);
    }

    #[test]
    fn horizontal_scroll_clamps_at_zero() {
        let mut v = view();
        v.scroll_left(10);
        assert_eq!(v.left, 0);
        v.scroll_right(16);
        assert_eq!(v.left, 16);
        v.scroll_left(8);
        assert_eq!(v.left, 8);
    }
}
