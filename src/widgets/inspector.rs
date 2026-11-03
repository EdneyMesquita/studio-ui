//! `.inspector`: the resizable right pane of a split (head with title, id and copy; scroll body; foot).
//!
//! Like Rabbit Studio's `Inspector` / `InspectorResizer`: one width shared by every view (380 by default, 280
//! to 640 by a drag; persisting it between runs is the app's job, through [`width`] and [`set_width`]), dragged from a 7 px grab area straddling its left border that fills
//! with the accent on hover; a double-click puts the default back. While an edge is dragged every inspector
//! on screen follows it (Rabbit's one store value); the width is saved on the release. A split pane narrower
//! than the width gives the table nothing and lays the inspector out at its full width from the pane's left,
//! its right part cut off at the pane's right edge (Rabbit's `shrink-0` inspector beside a `min-w-0 flex-1`
//! table).
//!
//! In a window up to 960 px wide it goes under its table instead, 45% of the view tall, with no splitter
//! (Rabbit's `@media (max-width: 960px)` hides it).

use egui::{Align, CursorIcon, Frame, Id, Key, Layout, Rect, Sense, Ui, UiBuilder, Vec2};

use crate::icons;
use crate::theme;

/// Windows up to this wide put every inspector under its table: Rabbit's `@media (max-width: 960px)`
/// (inclusive, on the window, not on the pane).
pub const STACK_UP_TO: f32 = 960.0;

/// The head's height; the head never shrinks (Rabbit's is `shrink-0`).
const HEAD_H: f32 = 36.0;

/// A stacked inspector's share of the view's height (Rabbit: `height: 45%`).
const STACKED_SHARE: f32 = 0.45;

/// The side inspector's width: Rabbit's `DEFAULT_INSPECTOR_WIDTH`, `MIN_INSPECTOR_WIDTH`,
/// `MAX_INSPECTOR_WIDTH`, and the step of an arrow key on a focused splitter.
pub const DEFAULT_WIDTH: f32 = 380.0;
pub const MIN_WIDTH: f32 = 280.0;
pub const MAX_WIDTH: f32 = 640.0;
const KEY_STEP: f32 = 16.0;

/// The grab area: 7 px, from 4 px left of the border (Rabbit: `-left-[4px] w-[7px]`).
const GRAB_W: f32 = 7.0;
const GRAB_LEFT: f32 = 4.0;

/// Where the shared width lives in egui's memory: every inspector reads it, a drag's release writes it.
fn width_id() -> Id {
    Id::new("studio-ui-inspector-width")
}

/// The drag of an edge under way, for every inspector: the grab area's id, the width at the press and the
/// pointer's x there.
fn drag_id() -> Id {
    Id::new("studio-ui-inspector-drag")
}

/// The width every side inspector takes (the user's, or the default).
pub fn width(ctx: &egui::Context) -> f32 {
    ctx.data(|d| d.get_temp::<f32>(width_id()))
        .unwrap_or(DEFAULT_WIDTH)
}

/// The width every side inspector shows this frame: while an edge is dragged (and on its release), where the
/// pointer has taken it, so all of them follow live like Rabbit's one store value; otherwise the shared width.
/// Worked out from the pointer, not from the dragged inspector, so a pane painted before it is not a frame late.
fn live_width(ctx: &egui::Context) -> f32 {
    let shared = width(ctx);
    let Some((grab, w0, x0)) = ctx.data(|d| d.get_temp::<(Id, f32, f32)>(drag_id())) else {
        return shared;
    };
    match ctx.read_response(grab) {
        Some(r) if r.dragged() || r.drag_stopped() => r
            .interact_pointer_pos()
            .map_or(shared, |p| drag_width(w0, x0, p.x)),
        _ => shared,
    }
}

/// Sets the shared width (a saved setting at start, a drag's release), clamped and rounded like Rabbit's. A
/// change repaints once, so the inspectors painted before the change this frame catch up.
pub fn set_width(ctx: &egui::Context, w: f32) {
    let w = clamp_width(w);
    if width(ctx) != w {
        ctx.data_mut(|d| d.insert_temp(width_id(), w));
        ctx.request_repaint();
    }
}

/// Rabbit's `clamp`: rounded, 280 to 640.
pub fn clamp_width(w: f32) -> f32 {
    if w.is_finite() {
        w.clamp(MIN_WIDTH, MAX_WIDTH).round()
    } else {
        DEFAULT_WIDTH
    }
}

/// Whether the inspectors in a window this wide go under their tables.
pub fn stacked(window_width: f32) -> bool {
    window_width <= STACK_UP_TO
}

/// A stacked inspector's height in a view `avail_h` tall, its foot `foot_h` tall last frame: 45% of the view,
/// never less than its head and foot (the body scrolls, down to nothing), never more than the view.
fn stacked_height(avail_h: f32, foot_h: f32) -> f32 {
    let avail_h = avail_h.max(0.0);
    (avail_h * STACKED_SHARE)
        .round()
        .max(HEAD_H + foot_h)
        .min(avail_h)
}

/// The room a side inspector takes beside its table in a view `avail_w` wide: its width, or the whole view
/// when that is narrower (the table gets the rest: nothing, then).
fn room_width(w: f32, avail_w: f32) -> f32 {
    w.min(avail_w.max(0.0))
}

/// A side inspector's rect, laid out in `panel` (the room it takes in the view: its own width, or the whole
/// view when that is narrower): always `w` wide, from the panel's left, so in a narrow pane the head and body
/// keep their layout and the right part is what the pane cuts off.
fn side_rect(panel: Rect, w: f32) -> Rect {
    Rect::from_min_size(panel.min, Vec2::new(w, panel.height()))
}

/// The width while dragging: the width at the press plus how far the pointer went left of where it pressed,
/// so the edge follows the pointer from wherever it was grabbed.
fn drag_width(start_w: f32, start_x: f32, x: f32) -> f32 {
    clamp_width(start_w - (x - start_x))
}

/// Lays out the foot at the bottom of `full`, `foot_h` tall (its height last frame) but never above `top`, in
/// its own clipped rect. Returns that rect and the height the foot wanted.
fn show_foot(
    ui: &mut Ui,
    id: Id,
    full: Rect,
    top: f32,
    foot_h: f32,
    foot: impl FnOnce(&mut Ui),
) -> (Rect, f32) {
    let rect = Rect::from_x_y_ranges(
        full.x_range(),
        (full.bottom() - foot_h).max(top)..=full.bottom(),
    );
    // as tall as its content wants (that is the measure), shown only inside its rect
    let inner = Rect::from_min_size(
        rect.min + Vec2::new(12.0, 7.0),
        Vec2::new((rect.width() - 24.0).max(0.0), 4096.0),
    );
    let mut child = ui.new_child(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        foot(ui);
    });
    (rect, child.min_rect().height() + 14.0)
}

#[derive(Clone, Copy, Debug)]
pub struct Inspector<'a> {
    id: Id,
    title: &'a str,
    id_text: Option<&'a str>,
    copy: Option<&'a str>,
    copy_tip: &'a str,
}

/// What happened in the inspector chrome this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct InspectorResponse {
    /// The copy button was clicked (the text is already on the clipboard).
    pub copied: bool,
}

impl<'a> Inspector<'a> {
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, title: &'a str) -> Self {
        Self {
            id: Id::new(id_salt),
            title,
            id_text: None,
            copy: None,
            copy_tip: "Copy",
        }
    }
    /// Faint mono id after the title (`3 · setpoint`).
    pub fn id_text(mut self, s: &'a str) -> Self {
        self.id_text = Some(s);
        self
    }
    /// Show a copy button that puts `text` on the clipboard.
    pub fn copy(mut self, text: &'a str) -> Self {
        self.copy = Some(text);
        self
    }

    /// The copy button's tooltip (`Copy`), in the app's language.
    pub fn copy_tip(mut self, tip: &'a str) -> Self {
        self.copy_tip = tip;
        self
    }

    pub fn show(self, ui: &mut Ui, body: impl FnOnce(&mut Ui)) -> InspectorResponse {
        self.show_impl(ui, body, None::<fn(&mut Ui)>)
    }

    pub fn show_with_foot(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui),
        foot: impl FnOnce(&mut Ui),
    ) -> InspectorResponse {
        self.show_impl(ui, body, Some(foot))
    }

    fn show_impl(
        self,
        ui: &mut Ui,
        body: impl FnOnce(&mut Ui),
        foot: Option<impl FnOnce(&mut Ui)>,
    ) -> InspectorResponse {
        let t = theme::theme(ui);
        let mut out = InspectorResponse::default();
        let avail = ui.available_rect_before_wrap();
        let stack = stacked(ui.ctx().content_rect().width());
        // the foot's height as it laid out last frame (none without a foot)
        let foot_h_id = self.id.with("foot-h");
        let foot_h = if foot.is_some() {
            ui.data(|d| d.get_temp::<f32>(foot_h_id)).unwrap_or(0.0)
        } else {
            0.0
        };
        // The splitter (side inspector only): resolved before the panel, so a drag moves it this frame.
        let grab_id = self.id.with("grab");
        let mut grab_active = false;
        if !stack && let Some(r) = ui.ctx().read_response(grab_id) {
            let ctx = ui.ctx();
            // a drag starts once the pointer moved: measure from where it was pressed, for every inspector
            if r.drag_started()
                && let Some(p) = ui.input(|i| i.pointer.press_origin())
            {
                let w0 = width(ctx);
                ctx.data_mut(|d| d.insert_temp(drag_id(), (grab_id, w0, p.x)));
            }
            grab_active = r.dragged();
            if r.drag_stopped() {
                // the release is the user's new width, for every inspector (saved by the app)
                set_width(ctx, live_width(ctx));
                ctx.data_mut(|d| d.remove::<(Id, f32, f32)>(drag_id()));
            } else if r.double_clicked() {
                set_width(ctx, DEFAULT_WIDTH);
            } else if r.has_focus() && !r.dragged() {
                let step = ui.input(|i| {
                    (i.key_pressed(Key::ArrowLeft) as i8 - i.key_pressed(Key::ArrowRight) as i8)
                        as f32
                });
                if step != 0.0 {
                    set_width(ctx, width(ctx) + step * KEY_STEP);
                }
            }
        }
        // the width shown: a drag's, wherever it is, or the shared one
        let live = live_width(ui.ctx());
        let (panel, size) = if stack {
            (
                egui::Panel::bottom(self.id.with("stacked")),
                stacked_height(avail.height(), foot_h),
            )
        } else {
            // the room it takes: its width, or the whole view (the table beside it gets nothing)
            (egui::Panel::right(self.id), room_width(live, avail.width()))
        };
        // head, body and foot, in the inspector's own rect
        let contents = |ui: &mut Ui| {
            let full = ui.max_rect();
            if stack {
                super::hline(ui.painter(), full.x_range(), full.top() + 0.5, t.line);
            } else {
                super::vline(ui.painter(), full.left() + 0.5, full.y_range(), t.line);
            }
            // the whole panel is the inspector's, whatever its parts take
            ui.expand_to_include_rect(full);
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            // head
            super::strip(ui, HEAD_H, None, Some(t.line_soft), (12.0, 6.0), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                super::text(ui, self.title, theme::semibold(theme::FS), t.text);
                if let Some(id) = self.id_text {
                    let w = (ui.available_width() - 32.0).max(0.0);
                    let g = super::galley_truncated(ui, id, theme::mono(theme::FS_XS), t.text_3, w);
                    let (r, _) = ui.allocate_exact_size(g.size(), egui::Sense::hover());
                    ui.painter().galley(r.min, g, t.text_3);
                }
                if let Some(text) = self.copy {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::icon_button(ui, icons::COPY, 24.0, self.copy_tip).clicked() {
                            ui.ctx().copy_text(text.to_owned());
                            out.copied = true;
                        }
                    });
                }
            });
            // Head, body and foot each get their own rect and draw only inside it, like Rabbit's flex
            // column (`shrink-0` head, `min-h-0 overflow-y-auto` body): a short inspector scrolls its body
            // and never paints it over the head or the foot.
            let body_top = ui.cursor().min.y.min(full.bottom());
            let mut body_bottom = full.bottom();
            if let Some(foot) = foot {
                let (rect, natural) = show_foot(ui, foot_h_id, full, body_top, foot_h, foot);
                body_bottom = rect.top();
                super::hline(ui.painter(), rect.x_range(), rect.top() + 0.5, t.line_soft);
                if (natural - foot_h).abs() > 0.5 {
                    ui.data_mut(|d| d.insert_temp(foot_h_id, natural));
                    // laid out again before anything shows, at its real height
                    ui.ctx().request_discard("inspector foot measured");
                }
            }
            let body_rect = Rect::from_x_y_ranges(full.x_range(), body_top..=body_bottom);
            // no room left between the head and the foot: nothing to scroll
            if body_rect.height() < 16.0 {
                return;
            }
            let mut body_ui = ui.new_child(
                UiBuilder::new()
                    .id_salt(self.id.with("body-ui"))
                    .max_rect(body_rect)
                    .layout(Layout::top_down(Align::Min)),
            );
            body_ui.set_clip_rect(body_rect.intersect(ui.clip_rect()));
            egui::ScrollArea::vertical()
                .id_salt(self.id.with("body"))
                .auto_shrink([false, false])
                .min_scrolled_height(0.0)
                .show(&mut body_ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.add_space(4.0);
                    body(ui);
                    ui.add_space(16.0);
                });
        };
        let shown = panel
            .exact_size(size)
            .resizable(false)
            .show_separator_line(false)
            .frame(Frame::NONE.fill(t.editor))
            .show(ui, |ui| {
                if stack {
                    contents(ui);
                    return;
                }
                // Beside the table: laid out at its full width from the panel's left, even when the pane is
                // narrower (the panel is then the whole pane), and shown only inside the panel, so its right
                // part is cut off at the pane's edge.
                let room = ui.max_rect();
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .id_salt(self.id.with("side"))
                        .max_rect(side_rect(room, live))
                        .layout(Layout::top_down(Align::Min)),
                );
                child.set_clip_rect(room.intersect(ui.clip_rect()));
                contents(&mut child);
                ui.expand_to_include_rect(room);
            });
        if !stack {
            // On top of the inspector's contents: the 7 px grab area across its left border.
            let edge = shown.response.rect.left();
            let rect = Rect::from_x_y_ranges(
                edge - GRAB_LEFT..=edge - GRAB_LEFT + GRAB_W,
                shown.response.rect.y_range(),
            );
            let r = super::tip(
                ui.interact(rect, grab_id, Sense::click_and_drag()),
                "Drag to resize - double-click to reset",
            );
            let on = r.hovered() || r.dragged() || grab_active;
            if on {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeColumn);
            }
            // Rabbit: `hover:bg-accent/60` fading in 150 ms, `focus-visible:bg-accent`
            let fade = ui.ctx().animate_bool_with_time(grab_id, on, 0.15);
            let a = if r.has_focus() { 1.0 } else { 0.6 * fade };
            if a > 0.0 {
                ui.painter().rect_filled(rect, 0, theme::alpha(t.accent, a));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_in_a_window_up_to_rabbits_breakpoint() {
        // the window decides, not the pane: any split at 1440 or 1280 keeps it beside its table
        assert!(!stacked(1440.0));
        assert!(!stacked(1280.0));
        assert!(!stacked(STACK_UP_TO + 1.0));
        // Rabbit's `max-width: 960px` includes 960; a small window down to the minimum: under it
        assert!(stacked(STACK_UP_TO));
        assert!(stacked(900.0));
    }

    #[test]
    fn stacked_is_45_percent_but_keeps_its_head_and_foot() {
        assert_eq!(stacked_height(700.0, 30.0), 315.0);
        assert_eq!(stacked_height(401.0, 30.0), 180.0);
        // 2×2 at 900×600: a view about 130 tall, a 55 px foot: head and foot, the body scrolls
        assert_eq!(stacked_height(130.0, 55.0), HEAD_H + 55.0);
        // a view shorter than head and foot: all of it
        assert_eq!(stacked_height(60.0, 55.0), 60.0);
        assert_eq!(stacked_height(-5.0, 0.0), 0.0);
    }

    #[test]
    fn the_width_is_rabbits() {
        assert_eq!(clamp_width(380.0), 380.0);
        assert_eq!(clamp_width(100.0), MIN_WIDTH);
        assert_eq!(clamp_width(900.0), MAX_WIDTH);
        assert_eq!(clamp_width(412.4), 412.0);
        assert_eq!(clamp_width(f32::NAN), DEFAULT_WIDTH);
        // the edge follows the pointer from where it was grabbed: 60 px left is 60 px wider
        assert_eq!(drag_width(380.0, 1062.0, 1002.0), 440.0);
        assert_eq!(drag_width(380.0, 1062.0, 1162.0), 280.0);
        assert_eq!(drag_width(380.0, 1062.0, 500.0), 640.0);
    }

    #[test]
    fn a_narrow_pane_keeps_the_full_width_from_its_left() {
        // the room it takes: its width, or a narrower view all of it (the table is left nothing)
        assert_eq!(room_width(380.0, 1140.0), 380.0);
        assert_eq!(room_width(380.0, 326.0), 326.0);
        assert_eq!(room_width(640.0, 560.0), 560.0);
        assert_eq!(room_width(380.0, -1.0), 0.0);
        let room =
            |x0: f32, w: f32| Rect::from_min_size(egui::pos2(x0, 100.0), Vec2::new(w, 500.0));
        // beside a table that has room: the panel is the inspector's width, and so is the inspector
        assert_eq!(side_rect(room(760.0, 380.0), 380.0), room(760.0, 380.0));
        // 3 columns at 1280 (a pane about 326 wide): the panel is the whole pane, the inspector still 380
        // from its left, the 54 px past the pane cut off
        assert_eq!(side_rect(room(300.0, 326.0), 380.0), room(300.0, 380.0));
        // a dragged 640 in a 560 pane: 640 from the pane's left
        assert_eq!(side_rect(room(300.0, 560.0), 640.0), room(300.0, 640.0));
        // a drag started in a narrow pane starts from the shared width, not from the pane's
        assert_eq!(drag_width(640.0, 300.0, 340.0), 600.0);
    }

    #[test]
    fn copy_tip_defaults_to_english_and_can_be_replaced() {
        assert_eq!(Inspector::new("i", "t").copy_tip, "Copy");
        assert_eq!(
            Inspector::new("i", "t").copy_tip("Copiar").copy_tip,
            "Copiar"
        );
    }
}
