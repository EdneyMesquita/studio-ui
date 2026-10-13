//! Line chart painter (same component as the siblings): several series, nice y ticks, `-Ns` time axis,
//! legend with current values, hover crosshair with a tooltip, dashed limit lines.
//! Returns before doing any work when its rect is not visible, so a chart scrolled away costs nothing.

use std::fmt::Write as _;

use egui::{
    Align2, Color32, Id, LayerId, Order, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind,
    Ui, Vec2,
};

use crate::theme::{self, Theme};

#[derive(Clone, Copy, Debug)]
pub struct Series<'a> {
    pub name: &'a str,
    pub color: Color32,
    /// Oldest first, newest last, one sample per `sample_secs`.
    pub values: &'a [f32],
    pub unit: &'a str,
    /// Dashed horizontal limit line in the series color.
    pub limit: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
pub struct Chart<'a> {
    id: Id,
    series: &'a [Series<'a>],
    sample_secs: f32,
    height: f32,
    decimals: Option<usize>,
    area: bool,
    legend: bool,
}

const PAD_L: f32 = 48.0;
const PAD_R: f32 = 6.0;
const PAD_T: f32 = 6.0;
const PAD_B: f32 = 18.0;
const LEGEND_H: f32 = 20.0;

/// `niceStep` of the mockup: 1, 2, 2.5, 5 × 10ⁿ.
pub fn nice_step(v: f64) -> f64 {
    if v <= 0.0 || !v.is_finite() {
        return 1.0;
    }
    let p = 10f64.powf(v.log10().floor());
    let n = v / p;
    let m = if n <= 1.0 {
        1.0
    } else if n <= 2.0 {
        2.0
    } else if n <= 2.5 {
        2.5
    } else if n <= 5.0 {
        5.0
    } else {
        10.0
    };
    m * p
}

/// `en` number format with thousands separators and fixed decimals: `4,153.5`, `-18.0`.
pub fn fmt_num(out: &mut String, v: f64, dp: usize) {
    if !v.is_finite() {
        out.push('—');
        return;
    }
    let mut tmp = String::with_capacity(24);
    let _ = write!(tmp, "{:.*}", dp, v.abs());
    let (int, frac) = tmp
        .split_once('.')
        .map_or((tmp.as_str(), ""), |(a, b)| (a, b));
    let negative = v < 0.0 && tmp.bytes().any(|b| b.is_ascii_digit() && b != b'0');
    if negative {
        out.push('-');
    }
    let len = int.len();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if !frac.is_empty() {
        out.push('.');
        out.push_str(frac);
    }
}

impl<'a> Chart<'a> {
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, series: &'a [Series<'a>]) -> Self {
        Self {
            id: Id::new(id_salt),
            series,
            sample_secs: 1.0,
            height: 150.0,
            decimals: None,
            area: false,
            legend: true,
        }
    }
    pub fn sample_secs(mut self, s: f32) -> Self {
        self.sample_secs = s;
        self
    }
    /// Plot height (the legend adds 20 px above).
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Decimals of legend and tooltip values (axis decimals follow the tick step).
    pub fn decimals(mut self, dp: usize) -> Self {
        self.decimals = Some(dp);
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let t = theme::theme(ui);
        let legend_h = if self.legend { LEGEND_H } else { 0.0 };
        let w = ui.available_width().max(120.0);
        let (rect, resp) =
            ui.allocate_exact_size(Vec2::new(w, legend_h + self.height), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return resp;
        }
        let n = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        let plot = Rect::from_min_max(
            egui::pos2(rect.left() + PAD_L, rect.top() + legend_h + PAD_T),
            egui::pos2(rect.right() - PAD_R, rect.bottom() - PAD_B),
        );
        // range and ticks
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for s in self.series {
            for v in s.values.iter().filter(|v| v.is_finite()) {
                lo = lo.min(*v as f64);
                hi = hi.max(*v as f64);
            }
        }
        if !lo.is_finite() {
            (lo, hi) = (0.0, 1.0);
        }
        if hi - lo < 1e-9 {
            hi += 1.0;
            lo -= 1.0;
        }
        let step = nice_step((hi - lo) / 3.0);
        lo = (lo / step).floor() * step;
        hi = (hi / step).ceil() * step;
        let ticks = ((hi - lo) / step).round() as usize;
        let axis_dp = if step >= 1.0 {
            0
        } else {
            (-step.log10()).ceil().min(3.0) as usize
        };
        let val_dp = self.decimals.unwrap_or(axis_dp);
        let x_of = |i: usize| plot.left() + plot.width() * i as f32 / (n.max(2) - 1) as f32;
        let y_of = |v: f64| plot.top() + plot.height() * (1.0 - ((v - lo) / (hi - lo)) as f32);

        let p = ui.painter_at(rect.expand(4.0));
        let mut buf = String::with_capacity(32);
        let axis_font = theme::mono(10.5);
        for k in 0..=ticks {
            let v = lo + step * k as f64;
            let y = y_of(v);
            super::hline(&p, plot.x_range(), y, t.line_soft);
            buf.clear();
            fmt_num(&mut buf, v, axis_dp);
            p.text(
                egui::pos2(plot.left() - 8.0, y),
                Align2::RIGHT_CENTER,
                &buf,
                axis_font.clone(),
                t.text_3,
            );
        }
        if n >= 2 {
            let mid = (n - 1) / 2;
            for (i, anchor) in [
                (0, Align2::LEFT_BOTTOM),
                (mid, Align2::CENTER_BOTTOM),
                (n - 1, Align2::RIGHT_BOTTOM),
            ] {
                buf.clear();
                time_label(&mut buf, (n - 1 - i) as f32 * self.sample_secs);
                p.text(
                    egui::pos2(x_of(i), rect.bottom() - 2.0),
                    anchor,
                    &buf,
                    axis_font.clone(),
                    t.text_3,
                );
            }
        }
        // series
        let stride = ((n as f32 / plot.width().max(1.0)).floor() as usize).max(1);
        for (si, s) in self.series.iter().enumerate() {
            if s.values.len() < 2 {
                continue;
            }
            let mut pts: Vec<Pos2> = Vec::with_capacity(s.values.len() / stride + 2);
            let off = n - s.values.len();
            let mut i = 0;
            while i < s.values.len() {
                pts.push(egui::pos2(x_of(off + i), y_of(s.values[i] as f64)));
                if i + 1 < s.values.len() && i + stride >= s.values.len() {
                    i = s.values.len() - 1;
                } else {
                    i += stride;
                }
            }
            if si == 0 && self.area {
                paint_area(&p, &pts, plot.bottom(), theme::alpha(s.color, 0.1));
            }
            p.add(Shape::line(pts, Stroke::new(1.75, s.color)));
            if let Some(lim) = s.limit {
                let y = y_of(lim as f64);
                if y >= plot.top() && y <= plot.bottom() {
                    p.extend(Shape::dashed_line(
                        &[egui::pos2(plot.left(), y), egui::pos2(plot.right(), y)],
                        Stroke::new(1.0, theme::alpha(s.color, 0.7)),
                        4.0,
                        3.0,
                    ));
                }
            }
        }
        if self.legend {
            self.paint_legend(ui, &p, rect, n, val_dp, t);
        }
        // hover crosshair + tooltip
        if n >= 2
            && let Some(pos) = resp
                .hover_pos()
                .filter(|p| plot.expand2(Vec2::new(0.0, PAD_T)).contains(*p))
        {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            let i = (((pos.x - plot.left()) / plot.width()) * (n - 1) as f32)
                .round()
                .clamp(0.0, (n - 1) as f32) as usize;
            let x = x_of(i);
            super::vline(&p, x, plot.y_range(), t.field_line);
            for s in self.series {
                let off = n - s.values.len();
                if let Some(v) = i.checked_sub(off).and_then(|j| s.values.get(j)) {
                    let c = egui::pos2(x, y_of(*v as f64));
                    p.circle(c, 3.5, s.color, Stroke::new(2.0, t.editor));
                }
            }
            self.paint_tip(ui, rect, x, plot.top(), i, n, val_dp, t);
        }
        resp
    }

    fn paint_legend(&self, ui: &Ui, p: &egui::Painter, rect: Rect, n: usize, dp: usize, t: &Theme) {
        let mut x = rect.left();
        let cy = rect.top() + 8.0;
        let mut buf = String::with_capacity(32);
        for (k, s) in self.series.iter().enumerate() {
            buf.clear();
            if let Some(v) = s.values.last().filter(|_| n > 0) {
                fmt_num(&mut buf, *v as f64, dp);
                if !s.unit.is_empty() {
                    buf.push(' ');
                    buf.push_str(s.unit);
                }
            }
            let g = super::galley(ui, s.name, theme::ui_font(theme::FS_XS), t.text_2);
            let v = super::galley(ui, buf.as_str(), theme::mono_medium(11.5), t.text);
            // a narrow chart names the series that fit and counts the rest
            if x + 16.0 + g.size().x + 6.0 + v.size().x > rect.right() {
                buf.clear();
                let _ = std::fmt::Write::write_fmt(
                    &mut buf,
                    format_args!("+{}", self.series.len() - k),
                );
                let more = super::galley(ui, buf.as_str(), theme::ui_font(theme::FS_XS), t.text_3);
                if x + more.size().x <= rect.right() {
                    p.galley(egui::pos2(x, cy - more.size().y / 2.0), more, t.text_3);
                }
                break;
            }
            p.rect_filled(
                Rect::from_center_size(egui::pos2(x + 5.0, cy), Vec2::new(10.0, 2.0)),
                2,
                s.color,
            );
            x += 16.0;
            let gw = g.size().x;
            p.galley(egui::pos2(x, cy - g.size().y / 2.0), g, t.text_2);
            x += gw + 6.0;
            let vw = v.size().x;
            p.galley(egui::pos2(x, cy - v.size().y / 2.0), v, t.text);
            x += vw + 14.0;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_tip(
        &self,
        ui: &Ui,
        rect: Rect,
        x: f32,
        top: f32,
        i: usize,
        n: usize,
        dp: usize,
        t: &Theme,
    ) {
        let tp = ui
            .ctx()
            .layer_painter(LayerId::new(Order::Tooltip, self.id.with("tip")));
        let mut buf = String::with_capacity(32);
        let secs = (n - 1 - i) as f32 * self.sample_secs;
        if secs > 0.0 {
            let _ = write!(buf, "{}s ago", secs.round() as i64);
        } else {
            buf.push_str("now");
        }
        let head = super::galley(ui, buf.as_str(), theme::mono(theme::FS_XS), t.text_3);
        let mut rows = Vec::with_capacity(self.series.len());
        let mut w = head.size().x.max(140.0 - 16.0);
        for s in self.series {
            let off = n - s.values.len();
            buf.clear();
            if let Some(v) = i.checked_sub(off).and_then(|j| s.values.get(j)) {
                fmt_num(&mut buf, *v as f64, dp);
                if !s.unit.is_empty() {
                    buf.push(' ');
                    buf.push_str(s.unit);
                }
            }
            let l = super::galley(ui, s.name, theme::ui_font(theme::FS_XS), t.text_2);
            let v = super::galley(ui, buf.as_str(), theme::mono(theme::FS_XS), t.text);
            w = w.max(16.0 + l.size().x + 10.0 + v.size().x);
            rows.push((s.color, l, v));
        }
        let h = 6.0 + head.size().y + 3.0 + rows.len() as f32 * 16.0 + 6.0;
        let mut left = x + 12.0;
        if left + w + 16.0 > rect.right() {
            left = x - 12.0 - w - 16.0;
        }
        let r = Rect::from_min_size(egui::pos2(left, top + 4.0), Vec2::new(w + 16.0, h));
        tp.add(t.shadow.as_shape(r, 6));
        tp.rect(r, 6, t.raised, Stroke::new(1.0, t.line), StrokeKind::Inside);
        let mut y = r.top() + 6.0;
        let hh = head.size().y;
        tp.galley(egui::pos2(r.left() + 8.0, y), head, t.text_3);
        y += hh + 3.0;
        for (c, l, v) in rows {
            let cy = y + 8.0;
            tp.rect_filled(
                Rect::from_center_size(egui::pos2(r.left() + 13.0, cy), Vec2::new(10.0, 2.0)),
                1,
                c,
            );
            tp.galley(
                egui::pos2(r.left() + 24.0, cy - l.size().y / 2.0),
                l,
                t.text_2,
            );
            tp.galley(
                egui::pos2(r.right() - 8.0 - v.size().x, cy - v.size().y / 2.0),
                v,
                t.text,
            );
            y += 16.0;
        }
    }
}

fn time_label(out: &mut String, secs: f32) {
    let s = secs.round() as i64;
    if s == 0 {
        out.push_str("now");
    } else if s >= 60 {
        let _ = write!(out, "-{}m", (s as f32 / 60.0).round() as i64);
    } else {
        let _ = write!(out, "-{s}s");
    }
}

fn paint_area(p: &egui::Painter, pts: &[Pos2], base: f32, color: Color32) {
    let mut mesh = egui::Mesh::default();
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(a, color);
        mesh.colored_vertex(b, color);
        mesh.colored_vertex(egui::pos2(b.x, base), color);
        mesh.colored_vertex(egui::pos2(a.x, base), color);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
    }
    p.add(Shape::mesh(mesh));
}

#[cfg(test)]
mod tests {
    use super::{fmt_num, nice_step};

    #[test]
    fn number_format() {
        let mut s = String::new();
        fmt_num(&mut s, 4153.5, 1);
        assert_eq!(s, "4,153.5");
        s.clear();
        fmt_num(&mut s, -18.0, 1);
        assert_eq!(s, "-18.0");
        s.clear();
        fmt_num(&mut s, -0.01, 1);
        assert_eq!(s, "0.0");
        s.clear();
        fmt_num(&mut s, 1234567.0, 0);
        assert_eq!(s, "1,234,567");
    }

    #[test]
    fn steps() {
        assert!((nice_step(0.7) - 1.0).abs() < 1e-9);
        assert!((nice_step(2.2) - 2.5).abs() < 1e-9);
        assert!((nice_step(34.0) - 50.0).abs() < 1e-9);
    }
}
