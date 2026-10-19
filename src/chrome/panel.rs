//! The side panel's frame (Rabbit `PanelHeader`, the mockup's `.sidepanel`): the background with its seam,
//! the 36 px head (title, a count, icon buttons), the search field, a faint note under the head, the
//! scrolling body and an optional foot note pinned at the bottom.
//!
//! ```no_run
//! # use studio_ui::chrome::{panel::{self, Body, Note}, tree};
//! # use studio_ui::{egui, icons, theme, widgets};
//! # fn f(ui: &mut egui::Ui, filter: &mut String) {
//! panel::background(ui);
//! panel::head(ui, "Explorer", |ui| {
//!     if widgets::icon_button(ui, icons::PLUS, theme::CONTROL, "New connection (Ctrl N)").clicked() {}
//! });
//! panel::search(ui, filter, "Search connections");
//! panel::body(ui, &Body::tree("explorer-tree"), |ui| {
//!     let _ = tree::node(ui, &tree::Row::new(0, tree::Lead::Icon(icons::BINARY), "Decoder"));
//! });
//! # }
//! ```
//!
//! Which panels exist, their titles and their buttons are the app's: the frame has no list of them.
//!
//! # How the apps map onto it
//!
//! | Mb Studio | This module |
//! |---|---|
//! | `explorer::side_panel` / `panels::side_panel`: panel fill, right seam, `item_spacing = 0` | [`background`] |
//! | the explorer's `strip(36, …)` head and `panels::head(title, count, right)` | [`head`]; the count is [`head_count`]`(ui, n, true)` called last in the buttons closure |
//! | the explorer's search row (8 px in, Escape clears, 8 px under) | [`search`] |
//! | the Register maps `.panel-note` (folder in mono 11 text-2, `· git main` faint, tooltip) | [`note`] with `Note { mono: true, muted: false, suffix: Some(..) }`, then `ui.add_space(4.0)` |
//! | `panels::empty` | [`empty`] |
//! | `ScrollArea::vertical().id_salt("explorer-tree")` with `add_space(2)` / `add_space(12)` | [`body`] with [`Body::tree`] |
//!
//! | OPC Studio | This module |
//! |---|---|
//! | `panels::side_panel` / `explorer::side_panel` background | [`background`] |
//! | `explorer::panel_head(title, buttons)` | [`head`]; the Address space's `"N nodes"` is [`head_count`]`(ui, text, false)` |
//! | the Servers search row | [`search`] (the Address space's own `Frame` with 8 × 4 margins stays the app's) |
//! | `panels::note(text, mono)` | [`note`] with `Note { mono, muted: true, suffix: None }` |
//! | `panels::empty` | [`empty`]: Rabbit's 1.45 line box, so OPC's single line moves down 1 px |
//!
//! New and opt-in: [`Body::foot`], the mockup's `.panel-foot`.

use egui::text::{LayoutJob, TextFormat};
use egui::{Rect, Response, Sense, Ui, UiBuilder, Vec2, pos2};

use crate::theme;
use crate::widgets::{self, field};

/// The head's height (`h-9`).
pub const HEAD_H: f32 = 36.0;
/// A note's height.
pub const NOTE_H: f32 = 22.0;

/// The panel's background: `panel` fill and the 1 px `seam` on its right edge. Also zeroes the item
/// spacing, as every part below expects. Call first, in the panel's own `Ui`.
pub fn background(ui: &mut Ui) {
    let t = theme::theme(ui);
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0, t.panel);
    widgets::vline(ui.painter(), rect.right() - 0.5, rect.y_range(), t.seam);
    ui.spacing_mut().item_spacing = Vec2::ZERO;
}

/// The head (`h-9 gap-0.5 pl-3 pr-1.5`): the title in semibold, then `buttons` laid out right to left (the
/// first one added is the rightmost), 2 px apart.
pub fn head(ui: &mut Ui, title: &str, buttons: impl FnOnce(&mut Ui)) {
    let t = theme::theme(ui);
    widgets::strip(ui, HEAD_H, None, None, (12.0, 6.0), |ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        widgets::text(ui, title, theme::semibold(theme::FS), t.text);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), buttons);
    });
}

/// A faint count in the head, left of the buttons added before it (`mr-1 text-xs text-fg-3`), in mono
/// (`tabular-nums`, a bare number) or in the UI font (`12 nodes`).
pub fn head_count(ui: &mut Ui, text: &str, mono: bool) {
    let t = theme::theme(ui);
    ui.add_space(4.0);
    let font = if mono {
        theme::mono(theme::FS_XS)
    } else {
        theme::ui_font(theme::FS_XS)
    };
    widgets::text(ui, text, font, t.text_3);
}

/// The search field under the head (`.panel-search`: `px-2 pb-2`). Escape clears it while it has focus
/// (Rabbit `ExplorerSearch`). Returns the text field's response.
pub fn search(ui: &mut Ui, buf: &mut String, hint: &str) -> Response {
    let resp = ui
        .horizontal(|ui| {
            ui.add_space(8.0);
            let resp = field::search_field(ui, buf, hint, ui.available_width() - 8.0);
            if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                buf.clear();
            }
            resp
        })
        .inner;
    ui.add_space(8.0);
    resp
}

/// A faint line under the head (`.panel-note`): a folder, the connection a panel shows.
#[derive(Clone, Copy, Debug, Default)]
pub struct Note<'a> {
    pub text: &'a str,
    /// In JetBrains Mono (paths, endpoints).
    pub mono: bool,
    /// In text-3; text-2 otherwise.
    pub muted: bool,
    /// A faint UI-font part after the text, 5 px from it (`· git main`).
    pub suffix: Option<&'a str>,
}

/// One 22 px [`Note`] line, `px-3`, cut with an ellipsis. Returns its response (for a tooltip).
pub fn note(ui: &mut Ui, n: &Note<'_>) -> Response {
    let t = theme::theme(ui);
    let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), NOTE_H), Sense::hover());
    let font = if n.mono {
        theme::mono(theme::FS_XS)
    } else {
        theme::ui_font(theme::FS_XS)
    };
    let color = if n.muted { t.text_3 } else { t.text_2 };
    let mut job = LayoutJob::default();
    job.append(n.text, 0.0, TextFormat::simple(font, color));
    if let Some(s) = n.suffix {
        job.append(
            s,
            5.0,
            TextFormat::simple(theme::ui_font(theme::FS_XS), t.text_3),
        );
    }
    job.wrap = egui::text::TextWrapping::truncate_at_width(r.width() - 24.0);
    let g = ui.painter().layout_job(job);
    ui.painter().galley(
        pos2(r.left() + 12.0, r.center().y - g.size().y / 2.0),
        g,
        color,
    );
    resp
}

/// Rabbit's empty panel line: `px-3 py-2.5 text-sm text-fg-3`, wrapping on 1.45 lines.
pub fn empty(ui: &mut Ui, s: &str) {
    let t = theme::theme(ui);
    let w = ui.available_width();
    let job = LayoutJob::simple(
        s.to_owned(),
        theme::ui_font(theme::FS_SM),
        t.text_3,
        w - 24.0,
    );
    let (g, dy) = widgets::css_text(ui, job, widgets::lh(theme::FS_SM));
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, g.size().y + 20.0), Sense::hover());
    ui.painter()
        .galley(pos2(r.left() + 12.0, r.top() + 10.0 + dy), g, t.text_3);
}

/// How [`body`] scrolls and what it adds around the content.
#[derive(Clone, Copy, Debug)]
pub struct Body<'a> {
    /// The scroll area's id salt (its offset is kept per salt).
    pub id_salt: &'a str,
    /// `.tree`'s `padding: 2px 0 12px` around the content.
    pub tree: bool,
    /// A faint wrapped note pinned under the body, over a hairline (`.panel-foot`).
    pub foot: Option<&'a str>,
}

impl<'a> Body<'a> {
    /// A plain scrolling body.
    pub fn new(id_salt: &'a str) -> Self {
        Self {
            id_salt,
            tree: false,
            foot: None,
        }
    }

    /// A body that holds tree rows: 2 px above them, 12 px under.
    pub fn tree(id_salt: &'a str) -> Self {
        Self {
            tree: true,
            ..Self::new(id_salt)
        }
    }

    #[must_use]
    pub fn foot(mut self, text: &'a str) -> Self {
        self.foot = Some(text);
        self
    }
}

/// The foot's height for a panel `width` wide and its galley: `padding: 8px 12px`, 1 px top border,
/// 11 px text on 1.45 lines.
fn foot_layout(ui: &Ui, text: &str, width: f32) -> (std::sync::Arc<egui::Galley>, f32, f32) {
    let t = theme::theme(ui);
    let job = LayoutJob::simple(
        text.to_owned(),
        theme::ui_font(theme::FS_XS),
        t.text_3,
        (width - 24.0).max(1.0),
    );
    let (g, dy) = widgets::css_text(ui, job, widgets::lh(theme::FS_XS));
    let h = 1.0 + 8.0 + g.size().y + 8.0;
    (g, dy, h)
}

/// The rest of the panel: a vertical scroll area that fills it (no item spacing), then the foot.
pub fn body<R>(ui: &mut Ui, b: &Body<'_>, add: impl FnOnce(&mut Ui) -> R) -> R {
    let avail = ui.available_rect_before_wrap();
    let foot = b.foot.map(|s| foot_layout(ui, s, avail.width()));
    let foot_h = foot.as_ref().map_or(0.0, |f| f.2);
    let area = Rect::from_min_max(avail.min, pos2(avail.right(), avail.bottom() - foot_h));
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(area)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let tree = b.tree;
    let inner = egui::ScrollArea::vertical()
        .id_salt(b.id_salt)
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            if tree {
                ui.add_space(2.0);
            }
            let r = add(ui);
            if tree {
                ui.add_space(12.0);
            }
            r
        })
        .inner;
    if let Some((g, dy, h)) = foot {
        let t = theme::theme(ui);
        let r = Rect::from_min_max(pos2(avail.left(), avail.bottom() - h), avail.max);
        widgets::hline(ui.painter(), r.x_range(), r.top() + 0.5, t.line);
        ui.painter()
            .galley(pos2(r.left() + 12.0, r.top() + 1.0 + 8.0 + dy), g, t.text_3);
    }
    ui.allocate_rect(avail, Sense::hover());
    inner
}
