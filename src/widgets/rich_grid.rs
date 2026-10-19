//! A virtualized grid like [`super::grid::Grid`] whose cells paint themselves: rich text runs (colored hex
//! bytes, underlined CRC), bars, status pills and small buttons. Same header, row height, hover, selection and
//! rules as `table.grid`; only the rows inside the viewport are laid out.
//!
//! ```ignore
//! let out = RichGrid::new("frames", &cols, rows.len()).selected(sel).show(ui, |c| {
//!     c.text(&label, theme::data(), t.text);
//! });
//! ```

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Align2, Color32, FontId, Id, Painter, Rect, Response, Sense, Ui, Vec2};

use super::grid::{Column, GridResponse, HEADER_H, ROW_H};
use crate::theme::{self, Theme, alpha};

/// Horizontal padding inside a cell.
pub const PAD: f32 = 10.0;

/// One cell being painted. `ui` is the scrolled content ui, so `ui.interact` on a sub-rect makes a button
/// that wins over the row click.
pub struct RichCell<'u> {
    pub ui: &'u mut Ui,
    pub rect: Rect,
    pub row: usize,
    pub col: usize,
    pub right: bool,
    pub id: Id,
}

impl std::fmt::Debug for RichCell<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichCell")
            .field("row", &self.row)
            .field("col", &self.col)
            .finish()
    }
}

impl RichCell<'_> {
    pub fn painter(&self) -> &Painter {
        self.ui.painter()
    }

    /// The cell without its horizontal padding.
    pub fn inner(&self) -> Rect {
        self.rect.shrink2(Vec2::new(PAD, 0.0))
    }

    /// Paint a layout job on one line, truncated to the cell, aligned by the column. Returns its width.
    pub fn job(&mut self, mut job: LayoutJob) -> f32 {
        let inner = self.inner();
        job.wrap = TextWrapping::truncate_at_width(inner.width().max(0.0));
        let g = self.ui.painter().layout_job(job);
        let w = g.size().x;
        let cy = self.rect.center().y;
        let x = if self.right {
            inner.right() - w
        } else {
            inner.left()
        };
        self.ui.painter().galley(
            egui::pos2(x, cy - g.size().y / 2.0),
            g,
            Color32::PLACEHOLDER,
        );
        w
    }

    /// Plain text in one font and color.
    pub fn text(&mut self, s: &str, font: FontId, color: Color32) -> f32 {
        let job = LayoutJob::simple_singleline(s.to_owned(), font, color);
        self.job(job)
    }

    /// Text then a second run in another color (`3 ` + `Read HR`).
    pub fn text2(&mut self, a: &str, ca: Color32, b: &str, cb: Color32, font: FontId) -> f32 {
        let mut job = LayoutJob::default();
        job.append(a, 0.0, TextFormat::simple(font.clone(), ca));
        job.append(b, 0.0, TextFormat::simple(font, cb));
        self.job(job)
    }

    /// `.st`: status icon in its color, then a word in the text color (left aligned).
    pub fn pill(&mut self, icon: &str, icon_color: Color32, word: &str, text: Color32) {
        let inner = self.inner();
        let cy = self.rect.center().y;
        theme::paint_icon(
            self.ui.painter(),
            egui::pos2(inner.left() + 6.0, cy),
            icon,
            theme::ICON_XS,
            icon_color,
        );
        let mut job =
            LayoutJob::simple_singleline(word.to_owned(), theme::ui_font(theme::FS_SM), text);
        job.wrap = TextWrapping::truncate_at_width((inner.width() - 17.0).max(0.0));
        let g = self.ui.painter().layout_job(job);
        self.ui.painter().galley(
            egui::pos2(inner.left() + 17.0, cy - g.size().y / 2.0),
            g,
            text,
        );
    }

    /// `.bar`: a 6 px track with a fill of `frac` (0..=1), `w` wide at the left of the cell; returns its right x.
    pub fn bar(&mut self, frac: f32, w: f32, color: Color32, t: &Theme) -> f32 {
        let inner = self.inner();
        let cy = self.rect.center().y;
        let track = Rect::from_min_size(egui::pos2(inner.left(), cy - 3.0), Vec2::new(w, 6.0));
        self.ui.painter().rect_filled(track, 3, alpha(t.text, 0.08));
        let fw = (w * frac.clamp(0.0, 1.0)).max(if frac > 0.0 { 2.0 } else { 0.0 });
        if fw > 0.0 {
            self.ui.painter().rect_filled(
                Rect::from_min_size(track.min, Vec2::new(fw, 6.0)),
                3,
                color,
            );
        }
        track.right()
    }

    /// A small flat button inside the cell (ghost look); `bordered` adds a hairline border.
    pub fn button(&mut self, icon: Option<&str>, label: &str, bordered: bool) -> Response {
        let t = theme::theme(self.ui);
        let font = theme::mono(theme::FS_SM);
        let g = self
            .ui
            .painter()
            .layout_no_wrap(label.to_owned(), font, t.text);
        let iw = if icon.is_some() {
            theme::ICON_XS + 5.0
        } else {
            0.0
        };
        let w = g.size().x + iw + 16.0;
        let inner = self.inner();
        let r = Rect::from_min_size(
            egui::pos2(inner.left() - 2.0, self.rect.center().y - 11.0),
            Vec2::new(w.min(self.rect.width() - 4.0), 22.0),
        );
        let resp = self.ui.interact(r, self.id.with("btn"), Sense::click());
        let hot = resp.hovered() || resp.is_pointer_button_down_on();
        let p = self.ui.painter();
        if hot {
            p.rect_filled(r, theme::R_SM, t.hover);
        }
        if bordered {
            p.rect_stroke(
                r,
                theme::R_SM,
                egui::Stroke::new(1.0, t.field_line),
                egui::StrokeKind::Inside,
            );
        }
        let mut x = r.left() + 8.0;
        if let Some(icon) = icon {
            theme::paint_icon(
                p,
                egui::pos2(x + theme::ICON_XS / 2.0, r.center().y),
                icon,
                theme::ICON_XS,
                t.text,
            );
            x += iw;
        }
        p.galley(egui::pos2(x, r.center().y - g.size().y / 2.0), g, t.text);
        if hot {
            self.ui
                .ctx()
                .set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        resp
    }
}

/// The builder.
pub struct RichGrid<'a> {
    id: Id,
    columns: &'a [Column<'a>],
    rows: usize,
    selected: Option<usize>,
    scroll_to: Option<usize>,
    row_bg: Option<&'a dyn Fn(usize) -> Option<Color32>>,
}

impl std::fmt::Debug for RichGrid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RichGrid")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .finish()
    }
}

impl<'a> RichGrid<'a> {
    pub fn new(
        id_salt: impl std::hash::Hash + std::fmt::Debug,
        columns: &'a [Column<'a>],
        rows: usize,
    ) -> Self {
        Self {
            id: Id::new(id_salt),
            columns,
            rows,
            selected: None,
            scroll_to: None,
            row_bg: None,
        }
    }
    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected = row;
        self
    }
    /// Scroll this row into view.
    pub fn scroll_to(mut self, row: Option<usize>) -> Self {
        self.scroll_to = row;
        self
    }
    /// A background tint per row (rows flagged by the view), under hover and selection.
    pub fn row_bg(mut self, f: &'a dyn Fn(usize) -> Option<Color32>) -> Self {
        self.row_bg = Some(f);
        self
    }

    pub fn show(self, ui: &mut Ui, mut cell: impl FnMut(&mut RichCell<'_>)) -> GridResponse {
        let t = theme::theme(ui);
        let outer = ui.available_rect_before_wrap();
        ui.allocate_rect(outer, Sense::hover());
        let mut out = GridResponse::default();

        let fixed: f32 = self.columns.iter().map(|c| c.width).sum();
        let n_flex = self.columns.iter().filter(|c| c.flex).count();
        let spare = (outer.width() - fixed).max(0.0);
        let mut widths = [0.0f32; 32];
        let ncols = self.columns.len().min(widths.len());
        for (i, c) in self.columns.iter().take(ncols).enumerate() {
            widths[i] = c.width
                + if c.flex && n_flex > 0 {
                    spare / n_flex as f32
                } else {
                    0.0
                };
        }
        let widths = &widths[..ncols];
        let content_w: f32 = widths.iter().sum();
        // a pane shorter than the header keeps it inside (the inspector may sit right under)
        let header = Rect::from_min_size(
            outer.min,
            Vec2::new(outer.width(), HEADER_H.min(outer.height())),
        );
        let body = Rect::from_min_max(egui::pos2(outer.left(), header.bottom()), outer.max);

        let mut body_ui = ui.new_child(egui::UiBuilder::new().max_rect(body).id_salt(self.id));
        body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
        body_ui.spacing_mut().item_spacing = Vec2::ZERO;
        let rows = self.rows;
        let selected = self.selected;
        let scroll_to = self.scroll_to;
        let id = self.id;
        let columns = self.columns;
        let row_bg_fn = self.row_bg;
        let scrolled = egui::ScrollArea::both()
            .id_salt(id.with("scroll"))
            .auto_shrink([false, false])
            .show_viewport(&mut body_ui, |ui, viewport| {
                ui.set_min_size(Vec2::new(content_w, rows as f32 * ROW_H));
                let origin = ui.max_rect().min;
                let first = ((viewport.min.y / ROW_H).floor().max(0.0) as usize).min(rows);
                let last = ((viewport.max.y / ROW_H).ceil().max(0.0) as usize).min(rows);
                let clip = ui.clip_rect();
                if let Some(r) = scroll_to.filter(|r| *r < rows) {
                    let rr = Rect::from_min_size(
                        egui::pos2(origin.x + viewport.min.x, origin.y + r as f32 * ROW_H),
                        Vec2::new(1.0, ROW_H),
                    );
                    ui.scroll_to_rect(rr, None);
                }
                for row in first..last {
                    out.laid_out += 1;
                    let top = origin.y + row as f32 * ROW_H;
                    let row_rect = Rect::from_min_size(
                        egui::pos2(origin.x, top),
                        Vec2::new(content_w.max(clip.width()), ROW_H),
                    );
                    let resp = ui.interact(
                        row_rect.intersect(clip),
                        id.with(("row", row)),
                        Sense::click(),
                    );
                    let hovered = resp.hovered();
                    if hovered {
                        out.hovered = Some(row);
                    }
                    if resp.clicked() {
                        out.clicked = Some(row);
                    }
                    if resp.secondary_clicked() {
                        out.secondary_clicked = Some(row);
                    }
                    if resp.double_clicked() {
                        out.double_clicked = Some((row, 0));
                    }
                    let painter = ui.painter().clone();
                    if let Some(bg) = row_bg_fn.and_then(|f| f(row)) {
                        painter.rect_filled(row_rect, 0, bg);
                    }
                    let is_sel = selected == Some(row);
                    if is_sel {
                        painter.rect_filled(row_rect, 0, t.sel_soft);
                    } else if hovered {
                        painter.rect_filled(row_rect, 0, t.row_hover);
                    }
                    let mut x = origin.x;
                    for (ci, w) in widths.iter().enumerate() {
                        let cr = Rect::from_min_size(egui::pos2(x, top), Vec2::new(*w, ROW_H));
                        x += w;
                        if !clip.intersects(cr) {
                            continue;
                        }
                        let mut c = RichCell {
                            ui: &mut *ui,
                            rect: cr,
                            row,
                            col: ci,
                            right: columns[ci].right,
                            id: id.with(("cell", row, ci)),
                        };
                        cell(&mut c);
                    }
                    super::hline(
                        &painter,
                        row_rect.x_range(),
                        row_rect.bottom() - 0.5,
                        t.line_soft,
                    );
                }
                let y0 = origin.y + first as f32 * ROW_H;
                let y1 = origin.y + last as f32 * ROW_H;
                let mut x = origin.x;
                let painter = ui.painter();
                for w in widths {
                    x += w;
                    super::vline(painter, x - 0.5, y0..=y1, t.line_soft);
                }
            });
        let off_x = scrolled.state.offset.x;

        let hp = ui.painter_at(header);
        hp.rect_filled(header, 0, t.panel);
        let mut x = header.left() - off_x;
        for (col, w) in self.columns.iter().zip(widths) {
            paint_head(
                ui,
                &hp,
                Rect::from_min_size(egui::pos2(x, header.top()), Vec2::new(*w, HEADER_H)),
                col,
                t,
            );
            x += w;
        }
        super::hline(&hp, header.x_range(), header.bottom() - 0.5, t.line);
        out
    }
}

fn paint_head(ui: &Ui, p: &Painter, r: Rect, col: &Column<'_>, t: &Theme) {
    let mut job = LayoutJob::default();
    job.append(
        col.title,
        0.0,
        TextFormat::simple(theme::medium(theme::FS_SM), t.text),
    );
    if let Some(h) = col.hint {
        job.append(h, 6.0, TextFormat::simple(theme::mono(10.5), t.text_3));
    }
    job.wrap = TextWrapping::truncate_at_width((r.width() - 20.0).max(0.0));
    let g = ui.painter().layout_job(job);
    let cy = r.center().y;
    let pos = if col.right {
        Align2::RIGHT_CENTER
            .anchor_size(egui::pos2(r.right() - PAD, cy), g.size())
            .min
    } else {
        Align2::LEFT_CENTER
            .anchor_size(egui::pos2(r.left() + PAD, cy), g.size())
            .min
    };
    p.galley(pos, g, t.text);
    super::vline(p, r.right() - 0.5, r.y_range(), t.line_soft);
}
