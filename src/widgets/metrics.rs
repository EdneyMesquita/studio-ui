//! `.metrics`: a strip of tonal cells with hairline rules (no KPI cards).

use egui::{Color32, Rect, Sense, Ui, Vec2};

use crate::theme;

#[derive(Clone, Copy, Debug)]
pub struct Metric<'a> {
    pub icon: Option<&'a str>,
    pub key: &'a str,
    pub value: &'a str,
    /// Value color (warn/danger for bad numbers); text color when `None`.
    pub value_color: Option<Color32>,
    pub detail: &'a str,
    /// Clickable cell (hover highlight, reported by [`metrics`]).
    pub link: bool,
}

impl<'a> Metric<'a> {
    pub fn new(key: &'a str, value: &'a str, detail: &'a str) -> Self {
        Self {
            icon: None,
            key,
            value,
            value_color: None,
            detail,
            link: false,
        }
    }
    pub fn icon(mut self, glyph: &'a str) -> Self {
        self.icon = Some(glyph);
        self
    }
    pub fn color(mut self, c: Color32) -> Self {
        self.value_color = Some(c);
        self
    }
}

pub const HEIGHT: f32 = 60.0;

/// Narrowest cell before the strip wraps to another row (Rabbit's `repeat(auto-fit, minmax(112px, 1fr))`,
/// a little wider for the mono values of a port or an endpoint).
pub const MIN_CELL: f32 = 136.0;

/// Cells per row and rows for `n` cells across `width`.
pub fn grid_shape(n: usize, width: f32) -> (usize, usize) {
    if n == 0 {
        return (0, 0);
    }
    let per_row = ((width / MIN_CELL).floor() as usize).clamp(1, n);
    (per_row, n.div_ceil(per_row))
}

/// Equal-width cells across the available width, wrapping to more rows in a narrow pane; returns the clicked
/// cell.
pub fn metrics(ui: &mut Ui, cells: &[Metric<'_>]) -> Option<usize> {
    let t = theme::theme(ui);
    let width = ui.available_width();
    let (per_row, rows) = grid_shape(cells.len(), width);
    let height = HEIGHT * rows.max(1) as f32;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    super::hline(ui.painter(), rect.x_range(), rect.bottom() - 0.5, t.line);
    if cells.is_empty() || !ui.is_rect_visible(rect) {
        return None;
    }
    let cw = width / per_row as f32;
    let mut clicked = None;
    for (i, m) in cells.iter().enumerate() {
        let (row, col) = (i / per_row, i % per_row);
        let r = Rect::from_min_size(
            egui::pos2(
                rect.left() + cw * col as f32,
                rect.top() + HEIGHT * row as f32,
            ),
            Vec2::new(cw, HEIGHT - 1.0),
        );
        if m.link {
            let cr = ui.interact(r, resp.id.with(i), Sense::click());
            if cr.hovered() {
                ui.painter().rect_filled(r, 0, t.row_hover);
            }
            if cr.clicked() {
                clicked = Some(i);
            }
        }
        if col + 1 < per_row && i + 1 < cells.len() {
            super::vline(ui.painter(), r.right() - 0.5, r.y_range(), t.line_soft);
        }
        if row + 1 < rows {
            super::hline(ui.painter(), r.x_range(), r.bottom() + 0.5, t.line_soft);
        }
        let x = r.left() + 12.0;
        let inner_w = cw - 24.0;
        let mut kx = x;
        if let Some(icon) = m.icon {
            theme::paint_icon(
                ui.painter(),
                egui::pos2(kx + 6.0, r.top() + 15.0),
                icon,
                theme::ICON_XS,
                t.text_2,
            );
            kx += 17.0;
        }
        let k = super::galley_truncated(
            ui,
            m.key,
            theme::ui_font(theme::FS_XS),
            t.text_2,
            inner_w - (kx - x),
        );
        ui.painter().galley(
            egui::pos2(kx, r.top() + 15.0 - k.size().y / 2.0),
            k,
            t.text_2,
        );
        let vc = m.value_color.unwrap_or(t.text);
        let v = super::galley_truncated(ui, m.value, theme::mono_medium(13.0), vc, inner_w);
        ui.painter()
            .galley(egui::pos2(x, r.top() + 31.0 - v.size().y / 2.0), v, vc);
        let d = super::galley_truncated(
            ui,
            m.detail,
            theme::ui_font(theme::FS_XS),
            t.text_3,
            inner_w,
        );
        ui.painter().galley(
            egui::pos2(x, r.top() + 47.0 - d.size().y / 2.0),
            d,
            t.text_3,
        );
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_wrap_to_more_rows_when_narrow() {
        assert_eq!(grid_shape(6, 1200.0), (6, 1));
        assert_eq!(grid_shape(6, 600.0), (4, 2));
        assert_eq!(grid_shape(6, 300.0), (2, 3));
        assert_eq!(grid_shape(6, 90.0), (1, 6));
        assert_eq!(grid_shape(0, 500.0), (0, 0));
        assert_eq!(grid_shape(3, 5000.0), (3, 1));
    }
}
