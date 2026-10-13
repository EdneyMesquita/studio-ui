//! `.toast`: a transient confirmation at the bottom right, Rabbit's sonner `Toaster` (bottom 36, right 16,
//! 356 wide): it slides up and fades in, stays 4 s (the timer waits while the pointer is on it), then
//! fades out sliding down. Repaints run only while it moves and at its deadline.

use std::time::Duration;

use egui::{Color32, Order, Rect, Sense, Stroke, Vec2};

use crate::icons;
use crate::theme;

/// sonner's `TOAST_LIFETIME`.
pub const LIFETIME: f64 = 4.0;
/// sonner's enter transition (`transform 400ms, opacity 400ms`, CSS `ease`).
const ENTER: f64 = 0.4;
/// sonner unmounts a dismissed toast after `TIME_BEFORE_UNMOUNT`, mid-way through its exit transition.
const EXIT: f64 = 0.2;
const WIDTH: f32 = 356.0;
/// Left of the text: border, `px-3.5`, the icon box (16, margins -3 and +4), `gap-2.5`.
const TEXT_X: f32 = 1.0 + 14.0 - 3.0 + 16.0 + 4.0 + 10.0;
/// The title's line: 13 px on Rabbit's 1.45.
const TITLE_LH: f32 = 18.85;

#[derive(Clone, Debug)]
pub struct Toast {
    pub title: String,
    pub sub: String,
    pub icon: &'static str,
    /// Icon color; `ok` when `None`.
    pub color: Option<Color32>,
    born: f64,
    /// When the pointer came onto it (the timer is paused since).
    paused: Option<f64>,
}

impl Toast {
    pub fn new(ctx: &egui::Context, title: impl Into<String>, sub: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sub: sub.into(),
            icon: icons::CIRCLE_CHECK,
            color: None,
            born: ctx.input(|i| i.time),
            paused: None,
        }
    }
    pub fn icon(mut self, glyph: &'static str, color: Option<Color32>) -> Self {
        self.icon = glyph;
        self.color = color;
        self
    }
}

/// CSS `ease`: `cubic-bezier(0.25, 0.1, 0.25, 1)`.
fn ease(x: f32) -> f32 {
    // solve the curve's x(t) = x by bisection, then take y(t)
    let bez = |t: f32, a: f32, b: f32| {
        3.0 * (1.0 - t) * (1.0 - t) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
    };
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        if bez(mid, 0.25, 0.25) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bez((lo + hi) / 2.0, 0.1, 1.0)
}

/// Show the toast in `slot` (if any) and clear it once gone.
pub fn show(ctx: &egui::Context, slot: &mut Option<Toast>) {
    let Some(toast) = slot.as_mut() else {
        return;
    };
    let now = ctx.input(|i| i.time);
    let screen = ctx.content_rect();
    let t = theme::Theme::of(ctx.theme());
    // the description wraps (`!text-xs`, 1.45 lines)
    let text_w = WIDTH - TEXT_X - 15.0;
    let desc = (!toast.sub.is_empty()).then(|| {
        let mut job = egui::text::LayoutJob::simple(
            toast.sub.clone(),
            theme::ui_font(theme::FS_XS),
            t.text_2,
            text_w,
        );
        job.wrap.max_rows = 4;
        let line = super::lh(theme::FS_XS);
        for sec in &mut job.sections {
            sec.format.line_height = Some(line);
        }
        let g = ctx.fonts_mut(|f| f.layout_job(job));
        let font_h = ctx.fonts_mut(|f| f.row_height(&theme::ui_font(theme::FS_XS)));
        (g, ((line - font_h) / 2.0).round())
    });
    // WebKit lays the 58.85 px of a one-line toast out as 58
    let height =
        (1.0 + 10.0 + TITLE_LH + desc.as_ref().map_or(0.0, |d| 2.0 + d.0.size().y) + 10.0 + 1.0)
            .floor();
    let rect = Rect::from_min_size(
        egui::pos2(
            screen.right() - 16.0 - WIDTH,
            screen.bottom() - 36.0 - height,
        ),
        Vec2::new(WIDTH, height),
    );
    // the timer waits while the pointer is on the toast
    let hovered = ctx.pointer_hover_pos().is_some_and(|p| rect.contains(p));
    match (hovered, toast.paused) {
        (true, None) => toast.paused = Some(now),
        (false, Some(at)) => {
            toast.born += now - at;
            toast.paused = None;
        }
        _ => {}
    }
    let age = toast.paused.unwrap_or(now) - toast.born;
    let left = LIFETIME - age;
    if left <= -EXIT {
        *slot = None;
        ctx.request_repaint();
        return;
    }
    // enter: up from its own height, 0 → 1; exit: down again, 1 → 0
    let (k, moving) = if age < ENTER {
        (ease((age / ENTER) as f32), true)
    } else if left < 0.0 {
        (1.0 - ease((-left / EXIT) as f32 * 0.5), true)
    } else {
        (1.0, false)
    };
    if moving {
        ctx.request_repaint();
    } else if toast.paused.is_none() {
        ctx.request_repaint_after(Duration::from_secs_f64(left.max(0.0)));
    }
    let offset = (1.0 - k) * height;
    egui::Area::new(egui::Id::new("studio-ui-toast"))
        .order(Order::Foreground)
        .fade_in(false)
        .fixed_pos(rect.min + Vec2::new(0.0, offset))
        .interactable(false)
        .show(ctx, |ui| {
            ui.multiply_opacity(k);
            let (r, _) = ui.allocate_exact_size(rect.size(), Sense::hover());
            let p = ui.painter();
            // `!rounded-lg !border-line !bg-popover !shadow-overlay`
            p.add(t.shadow.as_shape(r, 8));
            p.rect(
                r,
                8,
                t.raised,
                Stroke::new(1.0, t.line),
                egui::StrokeKind::Inside,
            );
            // `!px-3.5 !py-2.5 !gap-2.5`; the icon box is 16 px with -3 / +4 px margins
            let icon_c = toast.color.unwrap_or(t.ok);
            theme::paint_icon(
                p,
                egui::pos2(r.left() + 1.0 + 14.0 - 3.0 + 8.0, r.center().y),
                toast.icon,
                theme::ICON,
                icon_c,
            );
            let x = r.left() + TEXT_X;
            // title: Rabbit's toasts draw it in the regular weight on the toast's 1.45 line (sonner's own
            // title rule does not reach it); description `!text-xs !text-fg-2`, 2 px under it
            let top = r.top() + 1.0 + 10.0;
            let tg =
                super::galley_truncated(ui, toast.title.as_str(), theme::body(), t.text, text_w);
            p.galley(
                egui::pos2(x, (top + TITLE_LH / 2.0 - tg.size().y / 2.0).round()),
                tg,
                t.text,
            );
            if let Some((dg, dy)) = desc {
                p.galley(egui::pos2(x, top + TITLE_LH + 2.0 + dy), dg, t.text_2);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::ease;

    #[test]
    fn css_ease_runs_from_zero_to_one() {
        assert!(ease(0.0).abs() < 1e-3);
        assert!((ease(1.0) - 1.0).abs() < 1e-3);
        // `ease` is ahead of linear at the middle
        assert!(ease(0.5) > 0.75);
    }
}
