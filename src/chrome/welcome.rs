//! What the editor shows with nothing open. The [`show`]n welcome screen (Rabbit `WelcomeScreen`, the
//! mockup's `.welcome`): the app's mark, name and version line, two columns at most 760 px wide (Start
//! actions with their keycaps, the Connect list), and three promises under a hairline. Also the
//! [`watermark`] (VS Code style shortcuts, Rabbit `ShortcutWatermark`) and [`pane_empty`] (a centred icon
//! tile, title and line).
//!
//! ```no_run
//! # use studio_ui::chrome::welcome::{self, Lead, Trail, Welcome, WelcomeRow};
//! # use studio_ui::{egui, icons, theme::identity};
//! # fn mark(_: &egui::Painter, _: egui::Rect) {}
//! # fn f(ui: &mut egui::Ui) {
//! let start = [WelcomeRow::new(Lead::Icon(icons::PLUS), "New connection").trail(Trail::Kbd("Ctrl N"))];
//! let connect = [WelcomeRow::new(Lead::Chip(identity::GREEN, "CR"), "Cold rooms")
//!     .trail(Trail::Detail { text: "Plant A · ", data: Some("ttyUSB0 · 19200 8E1"), color: None })];
//! let w = Welcome {
//!     mark: &mark,
//!     title: "Studio",
//!     version: "Version 0.1.0 · one client for every device",
//!     start: &start,
//!     connect: &connect,
//!     connect_empty: "No saved connections yet.",
//!     promises: [(icons::ACTIVITY, "The wire, visible"), (icons::LOCK, "Live plants ask first"), (icons::GIT_COMMIT_HORIZONTAL, "Files you can keep in git")],
//! };
//! let picked = welcome::show(ui, &w);
//! if picked.start == Some(0) { /* New connection */ }
//! # }
//! ```
//!
//! # How the apps map onto it
//!
//! | Mb Studio `views/welcome.rs` | This module |
//! |---|---|
//! | `show(ui, conns, eng) -> Option<Pick>` | [`show`]`(ui, &Welcome { .. })` → [`WelcomeResponse`] (`start` index → `Pick`, `connect` index → `Pick::Connect`) |
//! | the four Start rows (the first says `Add your first connection` with no connection) | four [`WelcomeRow`]s with [`Lead::Icon`] and [`Trail::Kbd`] |
//! | a connection: chip, name, `site · ` + the address in data font; the error in danger; the spinner while it connects (disabled) | [`Lead::Chip`], [`Trail::Detail`] (`color: Some(danger)` for the error), [`Trail::Spinner`] with `.enabled(false)` |
//! | the empty Connect text | [`Welcome::connect_empty`] |
//! | the `Start` and `Connect` heads in another language | [`show_with`] and [`WelcomeText`] |
//! | `PROMISES` and the logo | [`Welcome::promises`], [`Welcome::mark`] |
//! | `layout`, `promise_columns` (`rows_left` was 4) | [`layout`] (with `n_start`), [`promise_columns`] |
//! | `watermark` | [`watermark`] with the same four rows |
//!
//! | OPC Studio | This module |
//! |---|---|
//! | `views/placeholder.rs` `empty_editor` / `pane_empty` (OPC has no welcome screen yet) | [`pane_empty`], unchanged; the welcome screen is ready for it |
//!
//! The mockup's welcome (five Start rows, Connect rows from several kinds of connection with a faint kind
//! and site, and an "All N connections" row with an icon and a keycap) is the same API: Connect rows take
//! any [`Lead`] and [`Trail`].

use std::sync::Arc;

use egui::{Align2, Color32, Galley, Painter, Rect, Sense, Ui, Vec2, pos2};

use crate::chrome::{NARROW, window_width};
use crate::icons;
use crate::theme::{self, Theme};
use crate::widgets;

const ROW: f32 = 34.0;
/// The `h1`: the title's line (22 px on 1.2), `mt-0.5`, the version's (13 px on 1.45); the mark is
/// centered on it.
const HEAD_H: f32 = 26.4 + 2.0 + 18.85;
/// A section's `h2`: `mb-2 text-sm` (12 px on 1.45, then 8 px).
const SECTION_H: f32 = 17.4 + 8.0;
/// `px-6 py-10` around the screen.
const PAD_X: f32 = 24.0;
const PAD_Y: f32 = 40.0;
/// The promises' columns: `gap-x-4.5`, `gap-y-3` when stacked, a 12 px icon and `gap-1.5`.
const PROMISE_GAP_X: f32 = 18.0;
const PROMISE_GAP_Y: f32 = 12.0;
const PROMISE_ICON: f32 = 12.0 + 6.0;

/// A row's lead.
#[derive(Clone, Copy, Debug)]
pub enum Lead<'a> {
    /// A 16 px icon in accent text.
    Icon(&'a str),
    /// The small (14 px) identity chip.
    Chip(Color32, &'a str),
}

/// What a row shows at its right.
#[derive(Clone, Copy, Debug)]
pub enum Trail<'a> {
    /// A keycap.
    Kbd(&'a str),
    /// `text-xs` (text-3 unless `color`), then an optional part in the data font (an address), cut at 55 %.
    Detail {
        text: &'a str,
        data: Option<&'a str>,
        color: Option<Color32>,
    },
    /// A turning loader in text-3 (it connects).
    Spinner,
}

/// One 34 px ghost row of the Start or Connect column.
#[derive(Clone, Copy, Debug)]
pub struct WelcomeRow<'a> {
    pub lead: Lead<'a>,
    pub label: &'a str,
    pub trail: Option<Trail<'a>>,
    /// Off: `opacity-45`, no click (while it connects).
    pub enabled: bool,
}

impl<'a> WelcomeRow<'a> {
    pub fn new(lead: Lead<'a>, label: &'a str) -> Self {
        Self {
            lead,
            label,
            trail: None,
            enabled: true,
        }
    }

    #[must_use]
    pub fn trail(mut self, trail: Trail<'a>) -> Self {
        self.trail = Some(trail);
        self
    }

    #[must_use]
    pub fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }
}

/// The welcome screen's content.
pub struct Welcome<'a> {
    /// Paints the app's mark into a 40 px square.
    pub mark: &'a dyn Fn(&Painter, Rect),
    pub title: &'a str,
    /// The line under the title (`Version 0.1.0 · …`).
    pub version: &'a str,
    pub start: &'a [WelcomeRow<'a>],
    pub connect: &'a [WelcomeRow<'a>],
    /// Shown, wrapping, when `connect` is empty.
    pub connect_empty: &'a str,
    /// Three (icon, text) under the hairline.
    pub promises: [(&'a str, &'a str); 3],
}

impl std::fmt::Debug for Welcome<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Welcome")
            .field("title", &self.title)
            .field("version", &self.version)
            .field("start", &self.start)
            .field("connect", &self.connect)
            .field("connect_empty", &self.connect_empty)
            .field("promises", &self.promises)
            .finish_non_exhaustive()
    }
}

/// The words the screen itself writes, apart from what [`Welcome`] carries. The default is the English the
/// kit has always shown, so [`show`] is unchanged; [`show_with`] takes another language's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WelcomeText<'a> {
    /// The head over the first column (`Start`).
    pub start_title: &'a str,
    /// The head over the second column (`Connect`).
    pub connect_title: &'a str,
}

impl Default for WelcomeText<'_> {
    fn default() -> Self {
        Self {
            start_title: "Start",
            connect_title: "Connect",
        }
    }
}

/// The row clicked, by column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WelcomeResponse {
    pub start: Option<usize>,
    pub connect: Option<usize>,
}

/// The welcome screen in the editor's remaining space; it scrolls when the window is too short.
pub fn show(ui: &mut Ui, w: &Welcome<'_>) -> WelcomeResponse {
    show_with(ui, w, &WelcomeText::default())
}

/// [`show`] with the column heads in `text` (the app's language).
pub fn show_with(ui: &mut Ui, w: &Welcome<'_>, text: &WelcomeText<'_>) -> WelcomeResponse {
    let avail = ui.available_rect_before_wrap();
    let window = window_width(ui);
    let promises = promises(ui, w, avail.width(), window);
    let need = layout(
        avail.width(),
        window,
        w.start.len(),
        w.connect.len(),
        promises.h,
    )
    .total_h
        + 2.0 * PAD_Y;
    if need <= avail.height() {
        ui.allocate_rect(avail, Sense::hover());
        return paint(ui, avail, window, w, text, &promises);
    }
    // a short window: the screen scrolls instead of running off the bottom
    egui::ScrollArea::vertical()
        .id_salt("welcome")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (r, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), need), Sense::hover());
            paint(ui, r, window, w, text, &promises)
        })
        .inner
}

/// The welcome screen's geometry for the editor's width, the window's and the rows of each column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub w: f32,
    pub two: bool,
    /// The grid's gap, between columns and between rows.
    pub gap: f32,
    pub col_w: f32,
    pub body_h: f32,
    pub total_h: f32,
}

/// `w-[min(760px,100%)]` inside `px-6`, centered; the columns stack (and the gaps shrink) in a window under
/// 960 px, like the siblings' `max-[960px]:grid-cols-1 max-[960px]:gap-7`.
pub fn layout(
    width: f32,
    window: f32,
    n_start: usize,
    n_connect: usize,
    promises_h: f32,
) -> Layout {
    let w = (width - 2.0 * PAD_X).clamp(0.0, 760.0);
    let two = window >= NARROW;
    let gap = if two { 48.0 } else { 28.0 };
    let col_w = if two { (w - gap) / 2.0 } else { w };
    let rows_left = n_start as f32;
    let rows_right = n_connect.max(1) as f32;
    // under the first section head: both columns side by side, or Start, the gap, Connect's head and rows
    let body_h = if two {
        rows_left.max(rows_right) * ROW
    } else {
        (rows_left + rows_right) * ROW + gap + SECTION_H
    };
    Layout {
        w,
        two,
        gap,
        col_w,
        body_h,
        // header, gap, section head, body, gap, hairline and `pt-4`, the promises
        total_h: HEAD_H + gap + SECTION_H + body_h + gap + 17.0 + promises_h,
    }
}

/// The promises laid out: each one's column left edge (from the grid's) and wrapped text, and the height.
struct Promises {
    items: Vec<(f32, Arc<Galley>)>,
    h: f32,
}

/// `grid-cols-[repeat(3,auto)] justify-between`, or one column in a narrow window; text wraps in its
/// column when the three do not fit.
fn promises(ui: &Ui, wl: &Welcome<'_>, width: f32, window: f32) -> Promises {
    let t = theme::theme(ui);
    let w = (width - 2.0 * PAD_X).clamp(0.0, 760.0);
    let font = theme::ui_font(12.0);
    let wrapped = |s: &str, at: f32| {
        let mut job = egui::text::LayoutJob::simple(s.to_owned(), font.clone(), t.text_2, at);
        job.wrap.max_width = at.max(1.0);
        ui.painter().layout_job(job)
    };
    if window < NARROW {
        let mut y = 0.0;
        let items = wl
            .promises
            .iter()
            .map(|(_, s)| {
                let g = wrapped(s, w - PROMISE_ICON);
                let top = y;
                y += g.size().y + PROMISE_GAP_Y;
                (top, g)
            })
            .collect();
        return Promises {
            h: y - PROMISE_GAP_Y,
            items,
        };
    }
    // min-content (the longest word) and max-content of each column
    let maxs = wl
        .promises
        .map(|(_, s)| PROMISE_ICON + wrapped(s, f32::INFINITY).size().x);
    let mins = wl
        .promises
        .map(|(_, s)| PROMISE_ICON + wrapped(s, 1.0).size().x);
    let cols = promise_columns(mins, maxs, w);
    let spare = (w - cols.iter().sum::<f32>()).max(2.0 * PROMISE_GAP_X) / 2.0;
    let mut x = 0.0;
    let mut h: f32 = 0.0;
    let items = wl
        .promises
        .iter()
        .zip(cols)
        .map(|((_, s), col)| {
            let g = wrapped(s, col - PROMISE_ICON);
            let left = x;
            x += col + spare;
            h = h.max(g.size().y);
            (left, g)
        })
        .collect();
    Promises { items, h }
}

/// CSS grid sizing of three `auto` columns in `w` with 18 px gaps: each starts at its min-content and
/// grows equally toward its max-content while there is room.
pub fn promise_columns(mins: [f32; 3], maxs: [f32; 3], w: f32) -> [f32; 3] {
    let mut cols = mins;
    let mut free = w - 2.0 * PROMISE_GAP_X - mins.iter().sum::<f32>();
    loop {
        let open = (0..3).filter(|&k| cols[k] < maxs[k]).count();
        if free <= 0.01 || open == 0 {
            return cols;
        }
        let share = free / open as f32;
        for k in 0..3 {
            if cols[k] < maxs[k] {
                let add = share.min(maxs[k] - cols[k]);
                cols[k] += add;
                free -= add;
            }
        }
    }
}

fn paint(
    ui: &mut Ui,
    avail: Rect,
    window: f32,
    wl: &Welcome<'_>,
    text: &WelcomeText<'_>,
    promises: &Promises,
) -> WelcomeResponse {
    let t = theme::theme(ui);
    let mut out = WelcomeResponse::default();
    let Layout {
        w,
        two,
        gap,
        col_w,
        body_h,
        total_h,
    } = layout(
        avail.width(),
        window,
        wl.start.len(),
        wl.connect.len(),
        promises.h,
    );
    let rows_left = wl.start.len() as f32;
    // `place-items-center` inside `py-10`
    let top = (avail.center().y - total_h / 2.0).max(avail.top() + PAD_Y);
    let left = avail.center().x - w / 2.0;

    // header (`flex items-center gap-3.5`): the mark centered on the name and version lines
    let logo = Rect::from_min_size(
        pos2(left, (top + (HEAD_H - 40.0) / 2.0).round()),
        Vec2::splat(40.0),
    );
    (wl.mark)(ui.painter(), logo);
    let p = ui.painter();
    p.text(
        pos2(left + 54.0, (top + 26.4 / 2.0).round()),
        Align2::LEFT_CENTER,
        wl.title,
        theme::semibold(22.0),
        t.text,
    );
    p.text(
        pos2(left + 54.0, (top + 26.4 + 2.0 + 18.85 / 2.0).round()),
        Align2::LEFT_CENTER,
        wl.version,
        theme::ui_font(13.0),
        t.text_2,
    );

    let y0 = top + HEAD_H + gap;
    let mut start = Rect::from_min_size(pos2(left, y0), Vec2::new(col_w, 0.0));
    let mut connect = if two {
        Rect::from_min_size(pos2(left + col_w + gap, y0), Vec2::new(col_w, 0.0))
    } else {
        Rect::from_min_size(
            pos2(left, y0 + SECTION_H + rows_left * ROW + gap),
            Vec2::new(col_w, 0.0),
        )
    };

    section(ui, t, start.min, text.start_title);
    start.min.y += SECTION_H;
    for (k, row) in wl.start.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(start.left(), start.top() + k as f32 * ROW),
            Vec2::new(col_w, ROW),
        );
        if action_row(ui, t, r, row) {
            out.start = Some(k);
        }
    }

    section(ui, t, connect.min, text.connect_title);
    connect.min.y += SECTION_H;
    if wl.connect.is_empty() {
        // `py-2 text-sm text-fg-3`, wrapping in its column
        let job = egui::text::LayoutJob::simple(
            wl.connect_empty.to_owned(),
            theme::ui_font(12.0),
            t.text_3,
            connect.width(),
        );
        let (g, dy) = widgets::css_text(ui, job, widgets::lh(12.0));
        ui.painter()
            .galley(pos2(connect.left(), connect.top() + 8.0 + dy), g, t.text_3);
    }
    for (k, row) in wl.connect.iter().enumerate() {
        let r = Rect::from_min_size(
            pos2(connect.left(), connect.top() + k as f32 * ROW),
            Vec2::new(col_w, ROW),
        );
        if action_row(ui, t, r, row) {
            out.connect = Some(k);
        }
    }

    // the three promises, under a hairline
    let fy = y0 + SECTION_H + body_h + gap;
    widgets::hline(ui.painter(), left..=left + w, fy, t.line_soft);
    // `border-t pt-4`; in a row the columns share the top, stacked each has its own
    let y = fy + 1.0 + 16.0;
    for ((glyph, _), (at, g)) in wl.promises.iter().zip(promises.items.iter()) {
        let (x, top) = if two { (left + at, y) } else { (left, y + at) };
        // `items-center`: the icon on the middle of its (maybe wrapped) text
        let cy = top + g.size().y / 2.0;
        theme::paint_icon(ui.painter(), pos2(x + 6.0, cy), glyph, 12.0, t.text_2);
        ui.painter()
            .galley(pos2(x + PROMISE_ICON, top), g.clone(), t.text_2);
    }
    out
}

fn section(ui: &Ui, t: &Theme, at: egui::Pos2, title: &str) {
    ui.painter().text(
        pos2(at.x, (at.y + 17.4 / 2.0).round()),
        Align2::LEFT_CENTER,
        title,
        theme::semibold(12.0),
        t.text_2,
    );
}

/// One 34 px ghost row; true when clicked.
fn action_row(ui: &mut Ui, t: &Theme, rect: Rect, row: &WelcomeRow<'_>) -> bool {
    let enabled = row.enabled;
    // the row bleeds 10 px left and right of its column, like the sibling's -mx-2.5
    let hit = rect.expand2(Vec2::new(10.0, 0.0));
    let resp = ui.interact(
        hit,
        // two saved connections can share a name (an import): the row's place tells them apart
        ui.id().with(("welcome", row.label, rect.top().to_bits())),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let p = ui.painter();
    if resp.hovered() && enabled {
        // a ghost `Button`: `rounded-md`, `hover:bg-row-hover`
        p.rect_filled(hit, theme::R, t.row_hover);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let cy = rect.center().y;
    // `disabled:opacity-45` while it connects
    let mut p = p.clone();
    if !enabled {
        p.set_opacity(0.45);
    }
    // `gap-2.5` after the lead: a 16 px icon, or the default (14 px) connection chip
    let lead_w = match row.lead {
        Lead::Icon(g) => {
            theme::paint_icon(&p, pos2(rect.left() + 8.0, cy), g, 16.0, t.accent_text);
            16.0
        }
        Lead::Chip(c, ini) => {
            let r = Rect::from_center_size(pos2(rect.left() + 7.0, cy), Vec2::splat(14.0));
            widgets::chip::paint_chip(&p, r, c, ini, widgets::chip::ChipSize::Sm);
            14.0
        }
    };
    let label_x = rect.left() + lead_w + 10.0;
    let g = widgets::galley(ui, row.label, theme::ui_font(13.0), t.text);
    let label_right = label_x + g.size().x;
    p.galley(pos2(label_x, cy - g.size().y / 2.0), g, t.text);
    match row.trail {
        Some(Trail::Kbd(k)) => {
            let g = widgets::galley(ui, k, theme::medium(10.5), t.text_2);
            let size = Vec2::new(g.size().x + 12.0, 18.0);
            let r = Rect::from_min_size(pos2(rect.right() - size.x, cy - 9.0), size);
            widgets::paint_kbd(&p, r, g, t.editor, t.line, t.text_2);
        }
        Some(Trail::Detail { text, data, color }) => {
            let c = color.unwrap_or(t.text_3);
            // `ml-auto max-w-[55%] truncate`
            let max = (rect.width() * 0.55)
                .min(rect.right() - label_right - 16.0)
                .max(40.0);
            let mut job = egui::text::LayoutJob::default();
            job.append(text, 0.0, egui::TextFormat::simple(theme::ui_font(11.0), c));
            if let Some(m) = data {
                job.append(m, 0.0, egui::TextFormat::simple(theme::data(), c));
            }
            job.wrap = egui::text::TextWrapping::truncate_at_width(max);
            let g = ui.painter().layout_job(job);
            p.galley(
                pos2(rect.right() - g.size().x, (cy - g.size().y / 2.0).round()),
                g,
                c,
            );
        }
        Some(Trail::Spinner) => {
            // one turn a second, repainted only while it shows
            let a = (ui.input(|i| i.time) * std::f64::consts::TAU).rem_euclid(std::f64::consts::TAU)
                as f32;
            let g = ui.painter().layout_no_wrap(
                icons::LOADER_CIRCLE.to_owned(),
                theme::icon_font(theme::ICON_SM),
                t.text_3,
            );
            let c = pos2(rect.right() - theme::ICON_SM / 2.0, cy);
            p.add(
                egui::epaint::TextShape::new(c - g.size() / 2.0, g, t.text_3)
                    .with_angle_and_anchor(a, Align2::CENTER_CENTER),
            );
            ui.ctx().request_repaint();
        }
        None => {}
    }
    resp.clicked()
}

/// Connected with nothing open: the ways to get somewhere, as `label` and its keys. One key is plain text
/// (`click it in the Explorer`); several are keycaps.
pub fn watermark(ui: &mut Ui, rows: &[(&str, &[&str])]) {
    let t = theme::theme(ui);
    let avail = ui.available_rect_before_wrap();
    ui.allocate_rect(avail, Sense::hover());
    if rows.is_empty() {
        return;
    }
    // `grid grid-cols-[auto_auto] items-center gap-x-5 gap-y-2.5`, the grid centered as a whole: the
    // labels right-aligned in the first column, the keys (`gap-1`) in the second
    let line = widgets::lh(13.0);
    let gap_y = 10.0;
    let keys_w = |keys: &[&str]| -> f32 {
        if keys.len() == 1 {
            widgets::galley(ui, keys[0], theme::ui_font(13.0), t.text_2)
                .size()
                .x
        } else {
            keys.iter()
                .map(|k| {
                    widgets::galley(ui, *k, theme::medium(10.5), t.text_2)
                        .size()
                        .x
                        + 12.0
                })
                .sum::<f32>()
                + 4.0 * keys.len().saturating_sub(1) as f32
        }
    };
    let label_w = rows
        .iter()
        .map(|(l, _)| {
            widgets::galley(ui, *l, theme::ui_font(13.0), t.text_2)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    let key_w = rows.iter().map(|(_, k)| keys_w(k)).fold(0.0, f32::max);
    let grid_w = label_w + 20.0 + key_w;
    let grid_h = rows.len() as f32 * line + (rows.len() - 1) as f32 * gap_y;
    let top = (avail.center().y - grid_h / 2.0).round();
    let split = (avail.center().x - grid_w / 2.0 + label_w).round();
    for (k, (label, keys)) in rows.iter().enumerate() {
        let cy = top + k as f32 * (line + gap_y) + line / 2.0;
        let g = widgets::galley(ui, *label, theme::ui_font(13.0), t.text_2);
        ui.painter().galley(
            pos2(split - g.size().x, (cy - g.size().y / 2.0).round()),
            g,
            t.text_2,
        );
        let mut x = split + 20.0;
        for key in keys.iter() {
            if keys.len() == 1 {
                let g = widgets::galley(ui, *key, theme::ui_font(13.0), t.text_2);
                ui.painter()
                    .galley(pos2(x, (cy - g.size().y / 2.0).round()), g, t.text_2);
            } else {
                let g = widgets::galley(ui, *key, theme::medium(10.5), t.text_2);
                let size = Vec2::new(g.size().x + 12.0, 18.0);
                let r = Rect::from_min_size(pos2(x, (cy - 9.0).round()), size);
                widgets::paint_kbd(ui.painter(), r, g, t.editor, t.line, t.text_2);
                x += size.x + 4.0;
            }
        }
    }
}

/// `.pane-empty`: a 40 px tile with an icon, a semibold title and a short paragraph, centered in the
/// remaining space. Returns the centre x and the paragraph's bottom, for buttons under it.
pub fn pane_empty(ui: &mut Ui, icon: &str, title: &str, line: &str) -> (f32, f32) {
    let t = theme::theme(ui);
    let area = ui.available_rect_before_wrap();
    ui.allocate_rect(area, Sense::hover());
    let w = 290.0_f32.min(area.width() - 32.0).max(120.0);
    let title_g = widgets::galley_truncated(
        ui,
        title,
        theme::semibold(theme::FS),
        t.text,
        area.width() - 32.0,
    );
    let mut job =
        egui::text::LayoutJob::simple(line.to_owned(), theme::ui_font(theme::FS_SM), t.text_2, w);
    job.halign = egui::Align::Center;
    job.wrap.max_width = w;
    let para = ui.painter().layout_job(job);
    let total = 40.0 + 8.0 + title_g.size().y + 8.0 + para.size().y;
    let mut y = (area.center().y - total / 2.0).round();
    let cx = area.center().x;
    let tile = Rect::from_center_size(pos2(cx, y + 20.0), Vec2::splat(40.0));
    ui.painter().rect_filled(tile, theme::R_LG, t.row_hover);
    theme::paint_icon(ui.painter(), tile.center(), icon, 20.0, t.text_3);
    y += 48.0;
    ui.painter().galley(
        Align2::CENTER_TOP
            .anchor_size(pos2(cx, y), title_g.size())
            .min,
        title_g.clone(),
        t.text,
    );
    y += title_g.size().y + 8.0;
    // centred text is anchored at its middle
    let bottom = y + para.size().y;
    ui.painter().galley(pos2(cx, y), para, t.text_2);
    (cx, bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mb Studio's own tests, with its four Start rows made explicit.
    #[test]
    fn columns_stack_in_a_narrow_window() {
        // 1440 with the 300 px Explorer: two columns 48 apart, 760 wide
        let l = layout(1100.0, 1440.0, 4, 2, 18.0);
        assert!(l.two);
        assert_eq!(l.w, 760.0);
        assert_eq!(l.col_w, (760.0 - 48.0) / 2.0);
        // a 960 window keeps two columns in what the Explorer leaves, 24 px of air each side
        let l = layout(620.0, 960.0, 4, 2, 34.0);
        assert!(l.two);
        assert_eq!(l.w, 572.0);
        // one pixel less and they stack, 28 apart (the Explorer floats: the editor is wide again)
        let l = layout(919.0, 959.0, 4, 2, 54.0);
        assert!(!l.two);
        assert_eq!(l.gap, 28.0);
        assert_eq!(l.w, 760.0);
        assert_eq!(l.col_w, 760.0);
        // a narrow editor gives the screen all but its padding
        assert_eq!(layout(560.0, 900.0, 4, 2, 54.0).w, 512.0);
        // stacked, more connections make it taller (the screen then scrolls)
        assert!(
            layout(560.0, 900.0, 4, 5, 54.0).total_h > layout(560.0, 900.0, 4, 2, 54.0).total_h
        );
    }

    #[test]
    fn promises_wrap_in_their_columns_when_short_of_room() {
        // room for all three: each at its full width
        let cols = promise_columns([60.0, 50.0, 70.0], [200.0, 120.0, 180.0], 760.0);
        assert_eq!(cols, [200.0, 120.0, 180.0]);
        // short: the room past the longest words is shared equally, a column stops at its full width
        let cols = promise_columns([60.0, 50.0, 70.0], [200.0, 120.0, 180.0], 460.0);
        assert!((cols.iter().sum::<f32>() - (460.0 - 36.0)).abs() < 0.01);
        assert_eq!(cols[1], 120.0);
        assert!((cols[0] - cols[2] - (60.0 - 70.0)).abs() < 0.01);
        // shorter than the words: nothing below them
        assert_eq!(
            promise_columns([60.0, 50.0, 70.0], [200.0, 120.0, 180.0], 100.0),
            [60.0, 50.0, 70.0]
        );
    }

    #[test]
    fn column_heads_default_to_english_and_can_be_replaced() {
        let d = WelcomeText::default();
        assert_eq!((d.start_title, d.connect_title), ("Start", "Connect"));
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mark = |_: &Painter, _: Rect| {};
        let w = Welcome {
            mark: &mark,
            title: "App",
            version: "v",
            start: &[],
            connect: &[],
            connect_empty: "none",
            promises: [("a", "a"), ("b", "b"), ("c", "c")],
        };
        let text = WelcomeText {
            start_title: "Comecar",
            connect_title: "Conectar",
        };
        let mut frame = ctx.run_ui(egui::RawInput::default(), |ui| {
            show(ui, &w);
            show_with(ui, &w, &text);
        });
        frame.textures_delta.clear();
    }

    #[test]
    fn the_taller_column_sets_the_body() {
        // two columns: five Start rows beside three connections
        let l = layout(1100.0, 1440.0, 5, 3, 18.0);
        assert_eq!(l.body_h, 5.0 * ROW);
        // an empty Connect column still holds one row (its text)
        assert_eq!(layout(1100.0, 1440.0, 0, 0, 18.0).body_h, ROW);
        // stacked: both columns, the gap and Connect's head
        let l = layout(560.0, 900.0, 5, 3, 18.0);
        assert_eq!(l.body_h, 8.0 * ROW + 28.0 + SECTION_H);
    }
}
