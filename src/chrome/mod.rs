//! The IDE frame of the family, protocol-neutral: the main [`toolbar`], the tool [`stripe`], the side
//! [`panel`] and its [`tree`] rows, the editor [`tabs`] row, the [`statusbar`] and the [`welcome`] screen.
//!
//! Every widget takes plain data (labels, icons, identity colours, flags) and returns one [`egui::Response`]
//! per thing that can be clicked, indexed like the input. The app attaches what is its own: the menus
//! (`menu::context`, `menu::dropdown`, `menu::dev_open` keys), tooltips with its own wording, and what a
//! click means. Nothing here names a protocol, a device or a server.
//!
//! The widgets were lifted from the two apps that drew this frame by hand, Mb Studio and OPC Studio. Every
//! field defaults to what both drew; where they differed, the difference is a field. Each module's docs
//! say how each app's code maps onto its API, so either app can switch with no visible change.
//!
//! What stays in the apps: the docking (the `egui::Panel`s, the side panel's resizer and its float under
//! [`NARROW`]), the editor's split panes, and every menu's entries.

pub mod panel;
pub mod statusbar;
pub mod stripe;
pub mod tabs;
pub mod toolbar;
pub mod tree;
pub mod welcome;

use egui::Ui;

/// The side panel's width until the user drags it (Rabbit `DEFAULT_SIDEBAR_WIDTH`).
pub const SIDE_DEFAULT: f32 = 300.0;
/// Rabbit `SidebarResizer` `MIN_WIDTH`.
pub const SIDE_MIN: f32 = 180.0;
/// Under this window width the side panel floats over the editor, the breadcrumb hides, the welcome
/// screen stacks and the status bar drops its secondary segments (Tailwind `max-[960px]`).
pub const NARROW: f32 = 960.0;
/// The tool stripe's width.
pub const STRIPE_W: f32 = 40.0;

/// The window's width in points: the family's breakpoints are on the window, not the panel.
pub fn window_width(ui: &Ui) -> f32 {
    ui.ctx().content_rect().width()
}

/// The side panel's width as shown: the saved width within 180 px and the smaller of 720 px and 60 % of
/// the window, rounded (Rabbit `clampSidebarWidth`). The saved value itself stays the user's.
pub fn clamp_side_width(width: f32, window: f32) -> f32 {
    let width = if width.is_finite() {
        width
    } else {
        SIDE_DEFAULT
    };
    let max = (window * 0.6).clamp(SIDE_MIN, 720.0);
    width.clamp(SIDE_MIN, max).round()
}

/// Two `min-w-0` flex items sharing `room` (CSS flex-shrink: each gives up room in proportion to its
/// own width, down to 0). Returns their widths; unchanged when both fit.
pub fn shares(a: f32, b: f32, room: f32) -> (f32, f32) {
    let total = a + b;
    if total <= room || total <= 0.0 {
        return (a, b);
    }
    let k = (room / total).max(0.0);
    (a * k, b * k)
}

/// [`shares`] for any number of items: when their sum exceeds `room`, each gives up width in proportion
/// to its own, down to 0.
pub fn shrink_all(widths: &mut [f32], room: f32) {
    let total: f32 = widths.iter().sum();
    if total <= room || total <= 0.0 {
        return;
    }
    let k = (room / total).max(0.0);
    for w in widths {
        *w *= k;
    }
}

/// Asks for one repaint `secs` from now, aimed just past that moment.
///
/// egui takes its predicted frame time off a requested delay so as not to overshoot; a delay near zero
/// then wakes the window early, the frame finds its deadline not yet reached, asks again and wakes at once
/// again, a burst of frames until the clock gets there. Adding the predicted frame time back (and 2 ms)
/// makes it land after.
pub fn repaint_after(ctx: &egui::Context, secs: f64) {
    let frame = f64::from(ctx.input(|i| i.predicted_dt).max(0.0));
    ctx.request_repaint_after(std::time::Duration::from_secs_f64(
        secs.max(0.0) + frame + 0.002,
    ));
}

/// Byte offset of the first ASCII-case-insensitive match of `needle` (already lowercase) in `hay`.
pub fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let at = hay.to_ascii_lowercase().find(needle)?;
    // ASCII lowercasing keeps byte offsets; the match must still fall on char boundaries
    (hay.is_char_boundary(at) && hay.is_char_boundary(at + needle.len())).then_some(at)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mb Studio's and OPC Studio's own tests, unchanged.
    #[test]
    fn side_width_fits_the_window_but_keeps_its_floor() {
        // 300 fits a 1440 window; 900 is capped at 720, then at 60 % of a small window
        assert_eq!(clamp_side_width(300.0, 1440.0), 300.0);
        assert_eq!(clamp_side_width(900.0, 1440.0), 720.0);
        assert_eq!(clamp_side_width(900.0, 960.0), 576.0);
        // never under 180, even when 60 % of the window is less
        assert_eq!(clamp_side_width(50.0, 1440.0), 180.0);
        assert_eq!(clamp_side_width(400.0, 200.0), 180.0);
        // rounded, and a broken saved value falls back to the default
        assert_eq!(clamp_side_width(250.4, 1440.0), 250.0);
        assert_eq!(clamp_side_width(f32::NAN, 1440.0), SIDE_DEFAULT);
    }

    #[test]
    fn shares_shrink_in_proportion() {
        // both fit: untouched
        assert_eq!(shares(100.0, 50.0, 200.0), (100.0, 50.0));
        // 60 too many: each gives up its share of the overflow
        let (a, b) = shares(100.0, 50.0, 90.0);
        assert!((a - 60.0).abs() < 1e-3 && (b - 30.0).abs() < 1e-3);
        // no room at all: both to zero, never negative
        assert_eq!(shares(100.0, 50.0, -10.0), (0.0, 0.0));
        assert_eq!(shares(0.0, 0.0, -10.0), (0.0, 0.0));
    }

    #[test]
    fn shrink_all_is_shares_for_any_count() {
        for (a, b, room) in [
            (100.0, 50.0, 200.0),
            (100.0, 50.0, 90.0),
            (100.0, 50.0, -10.0),
        ] {
            let mut w = [a, b];
            shrink_all(&mut w, room);
            let (x, y) = shares(a, b, room);
            assert!((w[0] - x).abs() < 1e-4 && (w[1] - y).abs() < 1e-4);
        }
        let mut w = [60.0, 30.0, 10.0];
        shrink_all(&mut w, 50.0);
        assert_eq!(w, [30.0, 15.0, 5.0]);
    }

    #[test]
    fn highlight_finds_ascii_case_insensitive_matches() {
        assert_eq!(find_ci("Cold rooms, line A", "line"), Some(12));
        assert_eq!(find_ci("Bench RS-485", "rs-4"), Some(6));
        assert_eq!(find_ci("Treatment SCADA", "scada"), Some(10));
        assert_eq!(find_ci("Bench", ""), None);
        assert_eq!(find_ci("Bench", "x"), None);
        // multi-byte text before the match keeps offsets valid
        assert_eq!(find_ci("Câmara 2", "2"), Some(8));
    }
}
