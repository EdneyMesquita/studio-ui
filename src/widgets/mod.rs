//! Reusable widgets of the family, painted by hand so they match the mockup in both themes
//! (egui's stock button/checkbox visuals are global and don't). Every view builds from these.

pub mod button;
pub mod chart;
pub mod check;
pub mod chip;
pub mod dialog;
pub mod field;
pub mod fit;
pub mod granted;
pub mod grid;
pub mod hexview;
pub mod inspector;
pub mod menu;
pub mod metrics;
pub mod notice;
pub mod palette;
pub mod rich_grid;
pub mod sec;
pub mod seg;
pub mod switch;
pub mod timeline;
pub mod toast;

use std::sync::Arc;

use egui::{
    Align2, Color32, FontId, Galley, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

use crate::theme::{self, Theme};

pub use button::{Btn, icon_button};
pub use chip::{Status, dot};
pub use fit::Fit;
pub use sec::kv;
pub use seg::{SegItem, seg, seg_fit, seg_stretch, seg_width};

/// Lay out one line of text (egui caches the galley across frames).
pub fn galley(ui: &Ui, text: impl Into<String>, font: FontId, color: Color32) -> Arc<Galley> {
    ui.painter().layout_no_wrap(text.into(), font, color)
}

/// One line of text truncated with an ellipsis at `max_width`.
pub fn galley_truncated(
    ui: &Ui,
    text: impl Into<String>,
    font: FontId,
    color: Color32,
    max_width: f32,
) -> Arc<Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.into(), font, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(0.0));
    ui.painter().layout_job(job)
}

/// A crisp 1 px horizontal rule.
pub fn hline(painter: &Painter, x: impl Into<egui::Rangef>, y: f32, color: Color32) {
    let y = painter.round_to_pixel_center(y);
    painter.hline(x, y, Stroke::new(1.0, color));
}

/// A crisp 1 px vertical rule.
pub fn vline(painter: &Painter, x: f32, y: impl Into<egui::Rangef>, color: Color32) {
    let x = painter.round_to_pixel_center(x);
    painter.vline(x, y, Stroke::new(1.0, color));
}

/// Plain text, allocated like a label (no wrapping).
pub fn text(ui: &mut Ui, s: impl Into<String>, font: FontId, color: Color32) -> Response {
    let g = galley(ui, s, font, color);
    let (rect, resp) = ui.allocate_exact_size(g.size(), Sense::hover());
    ui.painter().galley(rect.min, g, color);
    resp
}

/// Size of a keycap around its label: Rabbit's `Kbd` is 18 px high with 6 px side padding.
pub fn kbd_size(g: &Galley) -> Vec2 {
    Vec2::new(g.size().x + 12.0, 18.0)
}

/// A keycap's rect, vertically centered on `cy`, its right edge at `right`.
pub fn kbd_rect_right(g: &Galley, right: f32, cy: f32) -> Rect {
    let s = kbd_size(g);
    Rect::from_min_size(Pos2::new(right - s.x, (cy - s.y / 2.0).round()), s)
}

/// The CSS line box of a family text size: Rabbit's `--text-*--line-height: 1.45`.
pub fn lh(size: f32) -> f32 {
    size * 1.45
}

/// `job` laid out like CSS text: rows `line_height` apart. Returns the galley and the y offset to
/// paint it at inside its box (the half-leading that centers the glyphs in each row).
pub fn css_text(ui: &Ui, mut job: egui::text::LayoutJob, line_height: f32) -> (Arc<Galley>, f32) {
    let mut font_h = 0.0f32;
    for s in &mut job.sections {
        font_h = font_h.max(ui.ctx().fonts_mut(|f| f.row_height(&s.format.font_id)));
        s.format.line_height = Some(line_height);
    }
    let g = ui.painter().layout_job(job);
    (g, ((line_height - font_h) / 2.0).max(0.0).round())
}

/// A paragraph wrapped at the available width, rows [`lh`] apart like Rabbit's `<p>`.
pub fn para(ui: &mut Ui, text: &str, font: FontId, color: Color32) -> Response {
    let w = ui.available_width();
    let line = lh(font.size);
    let job = egui::text::LayoutJob::simple(text.to_owned(), font, color, w);
    let (g, dy) = css_text(ui, job, line);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, g.size().y), Sense::hover());
    ui.painter().galley(rect.min + Vec2::new(0.0, dy), g, color);
    resp
}

/// One line of text in a CSS line box `line_height` tall (a `Label`'s `leading-[1.45]`), allocated
/// like a label.
pub fn text_lh(ui: &mut Ui, s: impl Into<String>, font: FontId, color: Color32) -> Response {
    let line = lh(font.size);
    let g = galley(ui, s, font, color);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(g.size().x, line), Sense::hover());
    let y = (rect.center().y - g.size().y / 2.0).round();
    ui.painter().galley(egui::pos2(rect.left(), y), g, color);
    resp
}

/// Where a tooltip sits, as Radix places it: the side of the trigger, centered on it, `gap` away. The
/// family's tooltip carries a hidden arrow that Radix still measures, so every side offset is 10 px
/// more than the one written in Rabbit's source.
#[derive(Clone, Copy, Debug)]
pub struct TipSide {
    pub align: egui::RectAlign,
    pub flip: egui::RectAlign,
    pub gap: f32,
    /// A `max-w-*` of the caller's; the window's width otherwise.
    pub max_w: f32,
}

impl TipSide {
    /// `side="bottom" sideOffset={4}`, the toolbar's (and the default here).
    pub const BOTTOM: Self = Self {
        align: egui::RectAlign::BOTTOM,
        flip: egui::RectAlign::TOP,
        gap: 14.0,
        max_w: f32::INFINITY,
    };
    /// `side="right" sideOffset={6}`, the tool stripe's.
    pub const RIGHT: Self = Self {
        align: egui::RectAlign::RIGHT,
        flip: egui::RectAlign::LEFT,
        gap: 16.0,
        max_w: f32::INFINITY,
    };
    /// `side="top" className="max-w-md"` (Radix's default offset, 0), the dialog footer's test result.
    pub const TOP_MD: Self = Self {
        align: egui::RectAlign::TOP,
        flip: egui::RectAlign::BOTTOM,
        gap: 10.0,
        max_w: 448.0 - 2.0 * 8.0 - 2.0,
    };
}

/// The family's tooltip (Rabbit `TooltipContent`): popover bg, 1 px line border, 6 px radius,
/// `px-2 py-1`, 12 px text on a 17.4 px line, overlay shadow, below the trigger; shown after the
/// global 400 ms delay. Use instead of `on_hover_text`.
pub fn tip(resp: Response, text: &str) -> Response {
    tip_at(resp, text, TipSide::BOTTOM, None)
}

/// [`tip`] on another side, with an optional faint suffix (a shortcut: `ml-2 text-fg-3`).
pub fn tip_at(resp: Response, text: &str, side: TipSide, suffix: Option<&str>) -> Response {
    if text.is_empty() {
        return resp;
    }
    let t = theme::Theme::of(resp.ctx.theme());
    let mut tt = egui::Tooltip::for_enabled(&resp);
    // `w-fit`, no max width of its own: as wide as the window lets it
    let max_w = (resp.ctx.content_rect().width() - 2.0 * 8.0 - 2.0).min(side.max_w);
    tt.popup = tt
        .popup
        .frame(
            egui::Frame::NONE
                .fill(t.raised)
                .stroke(Stroke::new(1.0, t.line))
                .corner_radius(theme::R)
                // the overlay shadow, its blur kept under the tooltip's height: epaint clamps a
                // wider blur to it and its corners then show seams
                .shadow(egui::epaint::Shadow {
                    blur: 14,
                    offset: [0, 6],
                    ..t.shadow
                })
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .align(side.align)
        .align_alternatives(std::slice::from_ref(&side.flip))
        .gap(side.gap)
        .width(max_w);
    tt.show(|ui| {
        ui.set_max_width(max_w);
        let font = theme::ui_font(theme::FS_SM);
        let mut job = egui::text::LayoutJob::default();
        job.append(text, 0.0, egui::TextFormat::simple(font.clone(), t.text));
        if let Some(s) = suffix {
            job.append(s, 8.0, egui::TextFormat::simple(font, t.text_3));
        }
        job.wrap.max_width = max_w;
        let (g, dy) = css_text(ui, job, lh(theme::FS_SM));
        let (rect, _) = ui.allocate_exact_size(g.size(), Sense::hover());
        ui.painter()
            .galley(rect.min + Vec2::new(0.0, dy), g, t.text);
    });
    resp
}

pub fn paint_kbd(
    painter: &Painter,
    rect: Rect,
    g: Arc<Galley>,
    bg: Color32,
    border: Color32,
    fg: Color32,
) {
    painter.rect(
        rect,
        theme::R_SM,
        bg,
        Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    // the family's keycap has a 2 px bottom edge (`border-b-2`)
    painter.hline(
        rect.x_range().shrink(2.0),
        painter.round_to_pixel_center(rect.bottom() - 1.5),
        Stroke::new(1.0, border),
    );
    let pos = Align2::CENTER_CENTER
        .anchor_size(rect.center() - Vec2::new(0.0, 0.5), g.size())
        .min;
    painter.galley(pos, g, fg);
}

/// A horizontal strip of fixed height (querybar, tabbar2, results head): fill, bottom rule, inner padding,
/// children laid out left to right and vertically centered. Use `ui.with_layout(right_to_left)` inside for the
/// trailing group.
pub fn strip<R>(
    ui: &mut Ui,
    height: f32,
    fill: Option<Color32>,
    bottom_line: Option<Color32>,
    pad: (f32, f32),
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, 0, fill);
    }
    if let Some(line) = bottom_line {
        hline(ui.painter(), rect.x_range(), rect.bottom() - 0.5, line);
    }
    let inner = Rect::from_min_max(
        Pos2::new(rect.left() + pad.0, rect.top()),
        Pos2::new(rect.right() - pad.1, rect.bottom() - 1.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(inner.intersect(ui.clip_rect()));
    add(&mut child)
}

/// WebKit's `:focus-visible` heuristic: focus shows its ring once the keyboard moved it (Tab, arrows),
/// not after a click or a focus a click led to (a dialog's autofocused button). Call once a frame.
pub fn track_focus_visible(ctx: &egui::Context) {
    let (keys, pointer) = ctx.input(|i| {
        let keys = i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::Tab
                        | egui::Key::ArrowUp
                        | egui::Key::ArrowDown
                        | egui::Key::ArrowLeft
                        | egui::Key::ArrowRight,
                    pressed: true,
                    ..
                }
            )
        });
        (keys, i.pointer.any_pressed())
    });
    if keys || pointer {
        ctx.data_mut(|d| d.insert_temp(focus_visible_id(), keys && !pointer));
    }
}

fn focus_visible_id() -> egui::Id {
    egui::Id::new("studio-ui-focus-visible")
}

/// Whether keyboard focus shows its ring now (see [`track_focus_visible`]).
pub fn focus_is_visible(ctx: &egui::Context) -> bool {
    focus_visible(ctx)
}

fn focus_visible(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(focus_visible_id()))
        .unwrap_or(false)
}

/// The base focus outline (`:focus-visible { outline: 2px solid accent; outline-offset: -1px }`) of a
/// control that sets no offset of its own (the dialog nav's tabs): 2 px across its edge.
pub fn focus_ring_inset(painter: &Painter, rect: Rect, radius: u8, t: &Theme) {
    if !focus_visible(painter.ctx()) {
        return;
    }
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(2.0, t.accent),
        egui::StrokeKind::Middle,
    );
}

/// Keyboard focus outline of buttons and boxes: Rabbit's `focus-visible:outline-2 outline-offset-1
/// outline-ring` (2 px accent, 1 px outside the control); only while focus is keyboard-visible.
pub fn focus_ring(painter: &Painter, rect: Rect, radius: u8, t: &Theme) {
    if !focus_visible(painter.ctx()) {
        return;
    }
    painter.rect_stroke(
        rect.expand(1.0),
        radius.saturating_add(1),
        Stroke::new(2.0, t.accent),
        egui::StrokeKind::Outside,
    );
}
