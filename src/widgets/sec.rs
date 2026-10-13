//! Inspector building blocks: `.sec-h` section head, `.kv` key/value row, `table.mini`.

use egui::{Align, Color32, Response, Sense, Ui, Vec2};

use crate::theme;

/// `.sec-h`: 30 px section head, semibold 11 px title, faint note; a hairline above unless `first`.
pub fn sec_h(ui: &mut Ui, title: &str, note: Option<&str>, first: bool) -> Response {
    sec_h_with(ui, title, note, first, |_| {})
}

/// [`sec_h`] with trailing content laid out right to left (small `tb_button`s, counts).
pub fn sec_h_with(
    ui: &mut Ui,
    title: &str,
    note: Option<&str>,
    first: bool,
    right: impl FnOnce(&mut Ui),
) -> Response {
    let t = theme::theme(ui);
    if !first {
        ui.add_space(6.0);
    }
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::hover());
    if !first {
        super::hline(ui.painter(), rect.x_range(), rect.top() + 0.5, t.line_soft);
    }
    let cy = rect.center().y;
    // a narrow pane ends a long title with an ellipsis
    let g = super::galley_truncated(
        ui,
        title,
        theme::semibold(theme::FS_XS),
        t.text_2,
        rect.width() - 24.0,
    );
    let tw = g.size().x;
    ui.painter().galley(
        egui::pos2(rect.left() + 12.0, cy - g.size().y / 2.0),
        g,
        t.text_2,
    );
    if let Some(note) = note {
        let n = super::galley(ui, note, theme::ui_font(theme::FS_XS), t.text_3);
        ui.painter().galley(
            egui::pos2(rect.left() + 18.0 + tw, cy - n.size().y / 2.0),
            n,
            t.text_3,
        );
    }
    let inner = rect.shrink2(Vec2::new(10.0, 0.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::right_to_left(Align::Center)),
    );
    right(&mut child);
    resp
}

/// `.kv`: `key: value` in JetBrains Mono with an optional faint type/note on the right; value wraps.
/// `depth` indents by 16 px per level.
pub fn kv(
    ui: &mut Ui,
    depth: usize,
    key: &str,
    value: &str,
    value_color: Color32,
    note: Option<&str>,
) -> Response {
    let t = theme::theme(ui);
    let width = ui.available_width();
    let left = 6.0 + depth as f32 * theme::INDENT + 16.0 + 4.0;
    let mut job = egui::text::LayoutJob::default();
    let f = theme::data();
    job.append(key, 0.0, egui::TextFormat::simple(f.clone(), t.syn_key));
    job.append(":", 0.0, egui::TextFormat::simple(f.clone(), t.syn_punct));
    job.append(value, 6.0, egui::TextFormat::simple(f, value_color));
    let ng = note.map(|n| super::galley(ui, n, theme::ui_font(10.5), t.text_3));
    let note_w = ng.as_ref().map_or(0.0, |g| g.size().x + 8.0);
    job.wrap.max_width = (width - left - 10.0 - note_w).max(40.0);
    let g = ui.painter().layout_job(job);
    let h = (g.size().y + 4.0).max(theme::ROW);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 0, t.row_hover);
    }
    ui.painter()
        .galley(egui::pos2(rect.left() + left, rect.top() + 2.0), g, t.text);
    if let Some(n) = ng {
        let pos = egui::pos2(
            rect.right() - 10.0 - n.size().x,
            rect.top() + 2.0 + (20.0 - n.size().y) / 2.0,
        );
        ui.painter().galley(pos, n, t.text_3);
    }
    resp
}
