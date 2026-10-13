//! `table.grid`: the virtualized data grid. Only the rows inside the viewport are laid out and painted, so
//! 5,000 rows cost the same as 40. Sticky 30 px header (painted after the body with this frame's horizontal
//! offset), optional sticky first column (`td.rn`), right-aligned numeric columns, type hints in the header
//! (`raw hex`), hover, selected row, click / double-click / right-click per row.
//!
//! ```ignore
//! let cols = [Column::new("addr", 56.0).right(), Column::new("name", 180.0), Column::new("raw", 90.0).hint("hex").right()];
//! let out = Grid::new("regs", &cols, rows.len()).selected(Some(sel)).sticky_first(true).show(ui, |c| {
//!     match c.col { 0 => { let _ = write!(c.text, "{}", rows[c.row].addr); c.color = t.text_3; } .. }
//! });
//! if let Some(r) = out.clicked { sel = r; }
//! ```

use egui::text::{LayoutJob, TextFormat, TextWrapping};
use egui::{Align2, Color32, Id, Rect, Sense, Stroke, Ui, Vec2};

use crate::theme::{self, Theme};

pub const HEADER_H: f32 = 30.0;
pub const ROW_H: f32 = theme::ROW;

/// A grid column. Widths are in points; a `flex` column takes the spare width.
#[derive(Clone, Copy, Debug)]
pub struct Column<'a> {
    pub title: &'a str,
    /// Type hint after the title in the header (`hex`, `dec`, `scale`).
    pub hint: Option<&'a str>,
    pub width: f32,
    pub right: bool,
    pub flex: bool,
}

impl<'a> Column<'a> {
    pub fn new(title: &'a str, width: f32) -> Self {
        Self {
            title,
            hint: None,
            width,
            right: false,
            flex: false,
        }
    }
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
    pub fn flex(mut self) -> Self {
        self.flex = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellFont {
    Mono,
    MonoMedium,
    Ui,
}

/// What the cell callback fills in. `text` and `extra` arrive cleared; write with `push_str` or `write!`.
#[derive(Debug)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
    pub text: String,
    pub color: Color32,
    pub font: CellFont,
    /// Icon before the text (or alone, e.g. the status column).
    pub icon: Option<(&'static str, Color32)>,
    /// Icon after the text (the writable pencil).
    pub trail_icon: Option<(&'static str, Color32)>,
    /// Second run after the text in its own color (`none` + ` 0`).
    pub extra: String,
    pub extra_color: Color32,
    /// Strike the text through (invalid values).
    pub strike: bool,
    /// Cell background (changed-value highlight).
    pub bg: Option<Color32>,
}

impl Cell {
    fn reset(&mut self, row: usize, col: usize, t: &Theme) {
        self.row = row;
        self.col = col;
        self.text.clear();
        self.extra.clear();
        self.color = t.text;
        self.extra_color = t.text_3;
        self.font = CellFont::Mono;
        self.icon = None;
        self.trail_icon = None;
        self.strike = false;
        self.bg = None;
    }
}

/// What happened this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct GridResponse {
    pub clicked: Option<usize>,
    /// Row and column under the pointer.
    pub double_clicked: Option<(usize, usize)>,
    pub secondary_clicked: Option<usize>,
    pub hovered: Option<usize>,
    /// Rows laid out this frame (for tests and benchmarks).
    pub laid_out: usize,
}

type RowMenu<'m> = &'m mut dyn FnMut(&mut Ui, usize);

/// The grid builder.
pub struct Grid<'a> {
    id: Id,
    columns: &'a [Column<'a>],
    rows: usize,
    selected: Option<usize>,
    sticky_first: bool,
    scroll_to: Option<usize>,
}

impl std::fmt::Debug for Grid<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid")
            .field("id", &self.id)
            .field("rows", &self.rows)
            .finish()
    }
}

impl<'a> Grid<'a> {
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
            sticky_first: false,
            scroll_to: None,
        }
    }
    pub fn selected(mut self, row: Option<usize>) -> Self {
        self.selected = row;
        self
    }
    /// Keep the first column (`addr`, row numbers) pinned while scrolling horizontally.
    pub fn sticky_first(mut self, sticky: bool) -> Self {
        self.sticky_first = sticky;
        self
    }
    /// Scroll so this row is visible.
    pub fn scroll_to(mut self, row: Option<usize>) -> Self {
        self.scroll_to = row;
        self
    }

    pub fn show(self, ui: &mut Ui, cell: impl FnMut(&mut Cell)) -> GridResponse {
        self.show_impl(ui, cell, None)
    }

    fn show_impl(
        self,
        ui: &mut Ui,
        mut cell: impl FnMut(&mut Cell),
        mut menu: Option<RowMenu<'_>>,
    ) -> GridResponse {
        let t = theme::theme(ui);
        let outer = ui.available_rect_before_wrap();
        ui.allocate_rect(outer, Sense::hover());
        let mut out = GridResponse::default();

        // column geometry: flex columns share the spare width
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
        let area = egui::ScrollArea::both()
            .id_salt(self.id.with("scroll"))
            .auto_shrink([false, false]);
        let scroll_to = self.scroll_to;
        let rows = self.rows;
        let selected = self.selected;
        let sticky = self.sticky_first && ncols > 1;
        let id = self.id;
        let scrolled = area.show_viewport(&mut body_ui, |ui, viewport| {
            ui.set_min_size(Vec2::new(content_w, rows as f32 * ROW_H));
            let origin = ui.max_rect().min;
            let first = ((viewport.min.y / ROW_H).floor().max(0.0) as usize).min(rows);
            let last = ((viewport.max.y / ROW_H).ceil().max(0.0) as usize).min(rows);
            let clip = ui.clip_rect();
            let painter = ui.painter().clone();
            let vis_left = origin.x + viewport.min.x;
            if let Some(r) = scroll_to.filter(|r| *r < rows) {
                let rr = Rect::from_min_size(
                    egui::pos2(vis_left, origin.y + r as f32 * ROW_H),
                    Vec2::new(1.0, ROW_H),
                );
                ui.scroll_to_rect(rr, None);
            }
            let mut c = Cell {
                row: 0,
                col: 0,
                text: String::with_capacity(64),
                color: t.text,
                font: CellFont::Mono,
                icon: None,
                trail_icon: None,
                extra: String::with_capacity(32),
                extra_color: t.text_3,
                strike: false,
                bg: None,
            };
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
                    let col = resp
                        .interact_pointer_pos()
                        .map_or(0, |p| column_at(widths, origin.x, vis_left, sticky, p.x));
                    out.double_clicked = Some((row, col));
                }
                if let Some(menu) = menu.as_mut() {
                    egui::Popup::context_menu(&resp).show(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        menu(ui, row);
                    });
                }
                let row_bg = if selected == Some(row) {
                    Some(t.sel_soft)
                } else if hovered {
                    Some(t.row_hover)
                } else {
                    None
                };
                if let Some(bg) = row_bg {
                    painter.rect_filled(row_rect, 0, bg);
                }
                let mut x = origin.x;
                for (ci, w) in widths.iter().enumerate() {
                    let cr = Rect::from_min_size(egui::pos2(x, top), Vec2::new(*w, ROW_H));
                    x += w;
                    if (sticky && ci == 0) || !clip.intersects(cr) {
                        continue;
                    }
                    c.reset(row, ci, t);
                    cell(&mut c);
                    paint_cell(ui, &painter, cr, &c, self.columns[ci].right, t);
                }
                super::hline(
                    &painter,
                    row_rect.x_range(),
                    row_rect.bottom() - 0.5,
                    t.line_soft,
                );
                if sticky {
                    let cr =
                        Rect::from_min_size(egui::pos2(vis_left, top), Vec2::new(widths[0], ROW_H));
                    painter.rect_filled(cr, 0, row_bg.unwrap_or(t.editor));
                    c.reset(row, 0, t);
                    if selected != Some(row) {
                        c.color = t.text_3;
                    }
                    cell(&mut c);
                    paint_cell(ui, &painter, cr, &c, self.columns[0].right, t);
                    super::hline(&painter, cr.x_range(), cr.bottom() - 0.5, t.line_soft);
                }
            }
            // column rules over the visible rows
            let y0 = origin.y + first as f32 * ROW_H;
            let y1 = origin.y + last as f32 * ROW_H;
            let mut x = origin.x;
            for (ci, w) in widths.iter().enumerate() {
                x += w;
                if sticky && ci == 0 {
                    super::vline(&painter, vis_left + w - 0.5, y0..=y1, t.line_soft);
                } else {
                    super::vline(&painter, x - 0.5, y0..=y1, t.line_soft);
                }
            }
        });
        let off_x = scrolled.state.offset.x;

        // header, painted last with this frame's offset: sticky and in sync
        let hp = ui.painter_at(header);
        hp.rect_filled(header, 0, t.panel);
        let mut x = header.left() - off_x;
        for (ci, (col, w)) in self.columns.iter().zip(widths).enumerate() {
            let cx = if sticky && ci == 0 { header.left() } else { x };
            x += w;
            if sticky && ci == 0 {
                continue;
            }
            paint_head(
                ui,
                &hp,
                Rect::from_min_size(egui::pos2(cx, header.top()), Vec2::new(*w, HEADER_H)),
                col,
                t,
            );
        }
        if sticky {
            let r = Rect::from_min_size(header.min, Vec2::new(widths[0], HEADER_H));
            hp.rect_filled(r, 0, t.panel);
            paint_head(ui, &hp, r, &self.columns[0], t);
        }
        super::hline(&hp, header.x_range(), header.bottom() - 0.5, t.line);
        out
    }
}

fn column_at(widths: &[f32], origin_x: f32, vis_left: f32, sticky: bool, px: f32) -> usize {
    if sticky && px < vis_left + widths.first().copied().unwrap_or(0.0) {
        return 0;
    }
    let mut x = origin_x;
    for (i, w) in widths.iter().enumerate() {
        x += w;
        if px < x {
            return i;
        }
    }
    widths.len().saturating_sub(1)
}

fn paint_head(ui: &Ui, p: &egui::Painter, r: Rect, col: &Column<'_>, t: &Theme) {
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
            .anchor_size(egui::pos2(r.right() - 10.0, cy), g.size())
            .min
    } else {
        Align2::LEFT_CENTER
            .anchor_size(egui::pos2(r.left() + 10.0, cy), g.size())
            .min
    };
    p.galley(pos, g, t.text);
    super::vline(p, r.right() - 0.5, r.y_range(), t.line_soft);
}

fn paint_cell(ui: &Ui, p: &egui::Painter, r: Rect, c: &Cell, right: bool, _t: &Theme) {
    if let Some(bg) = c.bg {
        p.rect_filled(r.shrink2(Vec2::new(0.0, 0.0)), 0, bg);
    }
    let font = match c.font {
        CellFont::Mono => theme::data(),
        CellFont::MonoMedium => theme::mono_medium(theme::FS_DATA),
        CellFont::Ui => theme::body(),
    };
    let cy = r.center().y;
    let pad = 10.0;
    if c.text.is_empty() && c.extra.is_empty() {
        if let Some((icon, color)) = c.icon {
            let x = if right {
                r.right() - pad - 6.0
            } else {
                r.left() + pad + 6.0
            };
            theme::paint_icon(p, egui::pos2(x, cy), icon, theme::ICON_XS, color);
        }
        return;
    }
    let lead = if c.icon.is_some() { 18.0 } else { 0.0 };
    let trail = if c.trail_icon.is_some() { 18.0 } else { 0.0 };
    let mut job = LayoutJob::default();
    let mut fmt = TextFormat::simple(font.clone(), c.color);
    if c.strike {
        fmt.strikethrough = Stroke::new(1.0, c.color);
    }
    job.append(&c.text, 0.0, fmt);
    if !c.extra.is_empty() {
        job.append(&c.extra, 0.0, TextFormat::simple(font, c.extra_color));
    }
    job.wrap = TextWrapping::truncate_at_width((r.width() - 2.0 * pad - lead - trail).max(0.0));
    let g = ui.painter().layout_job(job);
    let gw = g.size().x;
    let x0 = if right {
        r.right() - pad - trail - gw - lead
    } else {
        r.left() + pad
    };
    if let Some((icon, color)) = c.icon {
        theme::paint_icon(p, egui::pos2(x0 + 6.0, cy), icon, theme::ICON_XS, color);
    }
    p.galley(egui::pos2(x0 + lead, cy - g.size().y / 2.0), g, c.color);
    if let Some((icon, color)) = c.trail_icon {
        theme::paint_icon(
            p,
            egui::pos2(x0 + lead + gw + 10.0, cy),
            icon,
            theme::ICON_XS,
            color,
        );
    }
}
