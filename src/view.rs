/// Viewport over a buffer of lines: which line is at the top of the screen
/// and how many rows/columns of content are visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    pub top: usize,
    pub height: usize,
    pub width: usize,
}

impl View {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            top: 0,
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
}
