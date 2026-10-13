//! Inputs: `.qfield` (labelled query-bar input and select), `.field` (search with icon) and `.input`.

use egui::{
    Align, Frame, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind, TextEdit, Ui, Vec2,
};

use crate::icons;
use crate::theme::{self, Theme, alpha};

/// Rabbit's `Input`: field bg, 1 px field-line border, no hover change; focused = accent border plus a 2 px
/// `ring-accent/30` outside it.
fn paint_box(ui: &Ui, idx: egui::layers::ShapeIdx, rect: Rect, t: &Theme, focused: bool) {
    paint_box_state(ui, idx, rect, t, focused, false);
}

/// [`paint_box`] that can be invalid (`aria-invalid:border-destructive aria-invalid:ring-destructive/25`):
/// a danger border, and the focus ring in danger too.
fn paint_box_state(
    ui: &Ui,
    idx: egui::layers::ShapeIdx,
    rect: Rect,
    t: &Theme,
    focused: bool,
    invalid: bool,
) {
    let border = if invalid {
        t.danger
    } else if focused {
        t.accent
    } else {
        t.field_line
    };
    let mut shapes = Vec::with_capacity(2);
    if focused {
        shapes.push(Shape::rect_filled(
            rect.expand(2.0),
            theme::R + 2,
            if invalid {
                alpha(t.danger, 0.25)
            } else {
                alpha(t.accent, 0.3)
            },
        ));
    }
    shapes.push(Shape::rect_filled(rect, theme::R, t.field));
    ui.painter().set(idx, Shape::Vec(shapes));
    ui.painter()
        .rect_stroke(rect, theme::R, Stroke::new(1.0, border), StrokeKind::Inside);
}

/// `.qfield`: a 30 px input with a lowercase label inside (`unit`, `range`, `every`); data in JetBrains Mono.
/// `width = f32::INFINITY` fills the remaining width. `right` right-aligns the value (numeric fields).
pub fn qfield(ui: &mut Ui, label: &str, buf: &mut String, width: f32, right: bool) -> Response {
    let t = theme::theme(ui);
    let width = if width.is_finite() {
        width
    } else {
        ui.available_width()
    };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::hover());
    let bg = ui.painter().add(Shape::Noop);
    let lg = super::galley(ui, label, theme::medium(theme::FS_XS), t.text_3);
    let lw = lg.size().x;
    ui.painter().galley(
        egui::pos2(rect.left() + 10.0, rect.center().y - lg.size().y / 2.0),
        lg,
        t.text_3,
    );
    let input = Rect::from_min_max(
        egui::pos2(rect.left() + 10.0 + lw + 8.0, rect.top() + 1.0),
        egui::pos2(rect.right() - 8.0, rect.bottom() - 1.0),
    );
    let resp = ui.put(
        input,
        TextEdit::singleline(buf)
            .frame(Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(theme::data())
            .text_color(t.text)
            .desired_width(input.width())
            .vertical_align(Align::Center)
            .horizontal_align(if right { Align::Max } else { Align::Min }),
    );
    paint_box(ui, bg, rect, t, resp.has_focus());
    resp
}

/// A `.qfield` that is a select: label, current option, chevron; a menu of `options` on click.
/// Returns `true` when `selected` changed.
pub fn qselect(
    ui: &mut Ui,
    label: &str,
    options: &[&str],
    selected: &mut usize,
    width: f32,
) -> bool {
    let t = theme::theme(ui);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
    let bg = ui.painter().add(Shape::Noop);
    let lg = super::galley(ui, label, theme::medium(theme::FS_XS), t.text_3);
    let lw = lg.size().x;
    let cy = rect.center().y;
    ui.painter().galley(
        egui::pos2(rect.left() + 10.0, cy - lg.size().y / 2.0),
        lg,
        t.text_3,
    );
    let cur = options.get(*selected).copied().unwrap_or("");
    let vg = super::galley_truncated(ui, cur, theme::data(), t.text, rect.width() - lw - 44.0);
    ui.painter().galley(
        egui::pos2(rect.left() + 18.0 + lw, cy - vg.size().y / 2.0),
        vg,
        t.text,
    );
    theme::paint_icon(
        ui.painter(),
        egui::pos2(rect.right() - 14.0, cy),
        icons::CHEVRON_DOWN,
        theme::ICON_XS,
        t.text_2,
    );
    let open = super::menu::is_open(&resp);
    paint_box(ui, bg, rect, t, open || resp.has_focus());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let mut changed = false;
    super::menu::dev_open(&resp, || format!("qselect:{label}"));
    super::menu::select_popup(&resp, Some(*selected), options.len(), |ui| {
        for (i, o) in options.iter().enumerate() {
            if super::menu::option(ui, o, *selected == i).clicked() && *selected != i {
                *selected = i;
                changed = true;
            }
        }
    });
    changed
}

/// `.field`: 28 px search input with a leading search icon and a hint.
pub fn search_field(ui: &mut Ui, buf: &mut String, hint: &str, width: f32) -> Response {
    let t = theme::theme(ui);
    let width = if width.is_finite() {
        width
    } else {
        ui.available_width()
    };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, theme::CONTROL), Sense::hover());
    let bg = ui.painter().add(Shape::Noop);
    theme::paint_icon(
        ui.painter(),
        egui::pos2(rect.left() + 15.0, rect.center().y),
        icons::SEARCH,
        theme::ICON_SM,
        t.text_3,
    );
    let input = Rect::from_min_max(
        egui::pos2(rect.left() + 28.0, rect.top() + 1.0),
        egui::pos2(rect.right() - 8.0, rect.bottom() - 1.0),
    );
    let resp = ui.put(
        input,
        TextEdit::singleline(buf)
            .frame(Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(theme::body())
            .text_color(t.text)
            .hint_text(RichText::new(hint).color(t.text_3).font(theme::body()))
            .desired_width(input.width())
            .vertical_align(Align::Center),
    );
    paint_box(ui, bg, rect, t, resp.has_focus());
    resp
}

/// `.input`: 32 px form input (dialogs), optionally monospace.
pub fn text_input(ui: &mut Ui, buf: &mut String, hint: &str, mono: bool, width: f32) -> Response {
    input_box(ui, buf, hint, mono, width, false, false)
}

/// [`text_input`] marked invalid (`aria-invalid`): danger border and focus ring.
pub fn text_input_invalid(
    ui: &mut Ui,
    buf: &mut String,
    hint: &str,
    mono: bool,
    width: f32,
) -> Response {
    input_box(ui, buf, hint, mono, width, false, true)
}

/// `.input` that shows dots instead of what is typed (passwords); `invalid` as [`text_input_invalid`].
pub fn password_input(
    ui: &mut Ui,
    buf: &mut String,
    hint: &str,
    width: f32,
    invalid: bool,
) -> Response {
    input_box(ui, buf, hint, false, width, true, invalid)
}

fn input_box(
    ui: &mut Ui,
    buf: &mut String,
    hint: &str,
    mono: bool,
    width: f32,
    password: bool,
    invalid: bool,
) -> Response {
    let t = theme::theme(ui);
    let width = if width.is_finite() {
        width
    } else {
        ui.available_width()
    };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, theme::FIELD), Sense::hover());
    let bg = ui.painter().add(Shape::Noop);
    let font = if mono { theme::data() } else { theme::body() };
    let input = rect.shrink2(Vec2::new(10.0, 1.0));
    let resp = ui.put(
        input,
        TextEdit::singleline(buf)
            .frame(Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(font.clone())
            .text_color(t.text)
            .hint_text(RichText::new(hint).color(t.text_3).font(font))
            .desired_width(input.width())
            .vertical_align(Align::Center)
            .password(password),
    );
    paint_box_state(ui, bg, rect, t, resp.has_focus(), invalid);
    resp
}
