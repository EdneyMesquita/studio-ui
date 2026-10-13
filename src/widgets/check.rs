//! Rabbit's `Checkbox`: a 14 px box, 3 px radius, field bg with a field-line border; checked fills accent
//! with a white 12 px check. Disabled dims to 50 %.

use egui::{Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::icons;
use crate::theme::{self, Theme};

/// Side of the box.
pub const SIDE: f32 = 14.0;

/// Paint the box centered on `center`.
pub fn paint(painter: &egui::Painter, center: egui::Pos2, checked: bool, t: &Theme) {
    let r = egui::Rect::from_center_size(center, Vec2::splat(SIDE));
    if checked {
        painter.rect_filled(r, theme::R_XS, t.accent);
        theme::paint_icon(
            painter,
            r.center(),
            icons::CHECK,
            theme::ICON_XS,
            t.on_accent,
        );
    } else {
        painter.rect(
            r,
            theme::R_XS,
            t.field,
            Stroke::new(1.0, t.field_line),
            StrokeKind::Inside,
        );
    }
}

/// A checkbox with a 13 px label 8 px after the box, on a 20 px row; toggles `checked` on click (label
/// included).
pub fn checkbox(ui: &mut Ui, checked: &mut bool, label: &str, enabled: bool) -> Response {
    checkbox_with(ui, checked, label, theme::body(), false, 20.0, enabled)
}

/// Rabbit's `<Label className="font-normal"><Checkbox/>…</Label>`: a 12 px label (`text-sm
/// leading-none`) 8 px after the box, as tall as the box; `muted` gives it `text-fg-2` (the import
/// list's "Select all").
pub fn checkbox_sm(
    ui: &mut Ui,
    checked: &mut bool,
    label: &str,
    muted: bool,
    enabled: bool,
) -> Response {
    checkbox_with(
        ui,
        checked,
        label,
        theme::ui_font(theme::FS_SM),
        muted,
        SIDE,
        enabled,
    )
}

fn checkbox_with(
    ui: &mut Ui,
    checked: &mut bool,
    label: &str,
    font: egui::FontId,
    muted: bool,
    height: f32,
    enabled: bool,
) -> Response {
    let t = theme::theme(ui);
    let color = if muted { t.text_2 } else { t.text };
    let g = (!label.is_empty()).then(|| super::galley(ui, label, font, color));
    let w = SIDE + g.as_ref().map_or(0.0, |g| 8.0 + g.size().x);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(w, height), sense);
    if enabled && resp.clicked() {
        *checked = !*checked;
        resp.mark_changed();
    }
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    // `disabled:opacity-50` dims the box; the label around it is not its `peer`, so it stays
    let mut p = ui.painter().clone();
    if !enabled {
        p.set_opacity(0.5);
    }
    let cy = rect.center().y;
    let center = egui::pos2(rect.left() + SIDE / 2.0, cy);
    paint(&p, center, *checked, t);
    if resp.has_focus() {
        super::focus_ring(
            &p,
            egui::Rect::from_center_size(center, Vec2::splat(SIDE)),
            theme::R_XS,
            t,
        );
    }
    if let Some(g) = g {
        ui.painter().galley(
            egui::pos2(rect.left() + SIDE + 8.0, (cy - g.size().y / 2.0).round()),
            g,
            color,
        );
    }
    resp
}
