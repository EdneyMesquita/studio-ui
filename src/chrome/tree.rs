//! Tree rows of the side panel (Rabbit `TreeRow`, the mockup's `.node`): 24 px, indented 16 px a level, a
//! twisty, an identity chip or icon, the label (optionally followed by a faint tag), the faint marks after
//! it and the row's "⋯" button.
//!
//! ```no_run
//! # use studio_ui::chrome::tree::{self, After, Lead, Row};
//! # use studio_ui::{egui, theme::identity, widgets::menu};
//! # fn f(ui: &mut egui::Ui) {
//! let r = tree::node(ui, &Row {
//!     open: Some(true),
//!     strong: true,
//!     tag: Some("kind"),
//!     after: &[After::Live(identity::GREEN)],
//!     more: true,
//!     ..Row::new(1, Lead::Chip(identity::TEAL, "WT"), "Water treatment")
//! });
//! if r.row.clicked() { /* toggle it */ }
//! tree::row_menu(&r, || "conn:wtp".into(), |ui| {
//!     menu::head(ui, "Water treatment");
//!     let _ = menu::item(ui, None, "Connect", None);
//! });
//! # }
//! ```
//!
//! The rows paint; walking the app's model (sites, connections, what is open, what matches the filter)
//! stays in the app. [`crate::chrome::find_ci`] finds the filter's match that [`Row::highlight`] picks out.
//!
//! # How the apps map onto it
//!
//! | Mb Studio `chrome/explorer.rs` | This module |
//! |---|---|
//! | `Lead::{Icon, Chip}`, `After::{Text, Mono, Hot, Warm, Icon, Live}` | the same variants |
//! | `Row { unit: Some(u8), .. }` | `Row { badge: Some(&unit.to_string()), .. }` |
//! | `node`, `NodeResp { row, more }`, `row_menu`, `find_ci` | [`node`], [`NodeResp`] (`link` is `None`), [`row_menu`], [`crate::chrome::find_ci`] |
//! | the "Nothing matches “…”." line under the tree | [`nothing_matches`] |
//!
//! | OPC Studio `chrome/explorer.rs` | This module |
//! |---|---|
//! | `After::{WarmText, Counted}`, `Row { mono, link, .. }`, `NodeResp { link, .. }` | the same names |
//! | `Row { unit: u8::try_from(sub.id).ok(), .. }` | `Row { badge: Some(&sub.id.to_string()), .. }` (the app keeps its `u8` cut-off) |
//!
//! New and opt-in, in neither app today: [`Row::tag`] (the mockup's faint protocol after a connection's
//! name, where several kinds of connection share one explorer), [`Row::guides`] (one hairline per level
//! under the parents' twisties) and [`Lead::Dot`] (the mockup's `.ldot`).

use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, Pos2, Rect, Response, Sense, Ui, Vec2, pos2};

use crate::icons;
use crate::theme::{self, Theme};
use crate::widgets::{self, chip::ChipSize, menu};

/// Leading glyph of a tree row.
#[derive(Clone, Copy, Debug)]
pub enum Lead<'a> {
    /// A 14 px icon in text-2.
    Icon(&'a str),
    /// The small (14 px) identity chip with its initials.
    Chip(Color32, &'a str),
    /// The mockup's `.ldot`: a 10 px ring in the colour over a 30 % tint of it.
    Dot(Color32),
}

impl Lead<'_> {
    /// Width of the slot the lead takes before the label's `gap-1.5`.
    pub fn width(&self) -> f32 {
        match self {
            // `.ldot`: 10 px and `margin: 0 3px`
            Lead::Dot(_) => 16.0,
            _ => 14.0,
        }
    }
}

/// Trailing marks of a tree row, painted right to left in the given order reversed.
#[derive(Clone, Copy, Debug)]
pub enum After<'a> {
    /// Faint 11 px text.
    Text(&'a str),
    /// Faint mono text (latency, paths).
    Mono(&'a str),
    /// Danger icon + word (`no reply`, `refused`).
    Hot(&'a str, &'a str),
    /// Warn icon + word (`alarm`).
    Warm(&'a str, &'a str),
    /// Warn-coloured word, no icon (`2 unacked`).
    WarmText(&'a str),
    /// A small icon in a colour.
    Icon(&'a str, Color32),
    /// The live dot.
    Live(Color32),
    /// A faint count after a small identity dot.
    Counted(Color32, &'a str),
}

/// What a tree row looks like; built per row, painted by [`node`].
#[derive(Clone, Copy, Debug)]
pub struct Row<'a> {
    pub depth: usize,
    /// `Some(open)` draws the twisty.
    pub open: Option<bool>,
    pub lead: Lead<'a>,
    /// A short mono badge before the label (an address or an id: `10`).
    pub badge: Option<&'a str>,
    pub label: &'a str,
    /// Faint mono text right after the label, cut with it (the mockup's protocol tag: `mb`, `ua`, `mq`).
    pub tag: Option<&'a str>,
    /// Medium weight (connections).
    pub strong: bool,
    /// The label is data: JetBrains Mono (file names).
    pub mono: bool,
    pub selected: bool,
    pub after: &'a [After<'a>],
    /// The row has a menu: reserve the "⋯" slot and show the button on hover.
    pub more: bool,
    /// A small accent button before the marks (`Review`).
    pub link: Option<&'a str>,
    /// The filter, lowercased: its first match in the label is picked out.
    pub highlight: &'a str,
    /// One hairline per level, under each parent's twisty.
    pub guides: bool,
}

impl<'a> Row<'a> {
    pub fn new(depth: usize, lead: Lead<'a>, label: &'a str) -> Self {
        Self {
            depth,
            open: None,
            lead,
            badge: None,
            label,
            tag: None,
            strong: false,
            mono: false,
            selected: false,
            after: &[],
            more: false,
            link: None,
            highlight: "",
            guides: false,
        }
    }
}

/// The row's response, and the "⋯" and link buttons' when the row has them.
#[derive(Debug)]
pub struct NodeResp {
    pub row: Response,
    pub more: Option<Response>,
    pub link: Option<Response>,
}

/// Where the parts of a row sit; see [`geom`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geom {
    /// The 16 px twisty box (painted only when the row has [`Row::open`]).
    pub twisty: Rect,
    /// The lead's slot.
    pub lead: Rect,
    /// The "⋯" button (20 px), when the row has a menu.
    pub more: Option<Rect>,
    /// Where the badge or the label starts.
    pub content_x: f32,
    /// Where the trailing part (link, marks) ends: the label is cut before what is painted from here leftward.
    pub marks_right: f32,
}

/// What a point of a row is over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Twisty,
    More,
    Row,
    Outside,
}

/// A row's geometry (Rabbit `TreeRow`): `pl 6 + 16 × depth`, a 16 px twisty box, `gap-1.5`, the lead,
/// `gap-1.5`, the content; the "⋯" (`icon-xs`, 20 px) `pr-2` from the right edge and laid out even while
/// hidden, so the marks never jump.
pub fn geom(rect: Rect, depth: usize, lead_w: f32, more: bool) -> Geom {
    let cy = rect.center().y;
    let x0 = rect.left() + 6.0 + depth as f32 * theme::INDENT;
    let twisty = Rect::from_min_size(pos2(x0, cy - 8.0), Vec2::splat(16.0));
    let lx = x0 + 16.0 + 6.0;
    let lead = Rect::from_min_size(pos2(lx, cy - 7.0), Vec2::new(lead_w, 14.0));
    let more = more
        .then(|| Rect::from_center_size(pos2(rect.right() - 8.0 - 10.0, cy), Vec2::splat(20.0)));
    Geom {
        twisty,
        lead,
        more,
        content_x: lx + lead_w + 6.0,
        marks_right: more.map_or(rect.right() - 8.0, |m| m.left() - 6.0),
    }
}

impl Geom {
    /// The part of the row under `p` (`rect` is the row's).
    pub fn hit(&self, rect: Rect, p: Pos2) -> Part {
        if !rect.contains(p) {
            Part::Outside
        } else if self.more.is_some_and(|m| m.contains(p)) {
            Part::More
        } else if self.twisty.contains(p) {
            Part::Twisty
        } else {
            Part::Row
        }
    }
}

/// x of the guide hairline of `level` (0-based) for a row starting at `left`: under that level's twisty.
pub fn guide_x(left: f32, level: usize) -> f32 {
    left + 6.0 + level as f32 * theme::INDENT + 7.5
}

/// One 24 px tree row. Hover is `row-hover`, selection `sel`.
pub fn node(ui: &mut Ui, row: &Row<'_>) -> NodeResp {
    let t = theme::theme(ui);
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), theme::ROW), Sense::click());
    let cy = rect.center().y;
    let g = geom(rect, row.depth, row.lead.width(), row.more);
    let more = g
        .more
        .map(|r| (r, ui.interact(r, resp.id.with("more"), Sense::click())));
    let hovered = ui.rect_contains_pointer(rect);
    if !ui.is_rect_visible(rect) {
        return NodeResp {
            row: resp,
            more: more.map(|m| m.1),
            link: None,
        };
    }
    let p = ui.painter();
    if row.selected {
        p.rect_filled(rect, 0, t.sel);
    } else if hovered {
        p.rect_filled(rect, 0, t.row_hover);
    }
    if row.guides {
        for level in 0..row.depth {
            widgets::vline(p, guide_x(rect.left(), level), rect.y_range(), t.line);
        }
    }
    if let Some(open) = row.open {
        let glyph = if open {
            icons::CHEVRON_DOWN
        } else {
            icons::CHEVRON_RIGHT
        };
        theme::paint_icon(p, g.twisty.center(), glyph, theme::ICON_XS, t.text_3);
    }
    match row.lead {
        Lead::Icon(glyph) => {
            theme::paint_icon(p, g.lead.center(), glyph, theme::ICON_SM, t.text_2);
        }
        Lead::Chip(c, initials) => widgets::chip::paint_chip(
            p,
            Rect::from_center_size(g.lead.center(), Vec2::splat(14.0)),
            c,
            initials,
            ChipSize::Sm,
        ),
        Lead::Dot(c) => {
            let center = g.lead.center();
            p.circle_filled(center, 5.0, theme::mix(c, 0.30, t.editor));
            p.circle_stroke(center, 4.25, egui::Stroke::new(1.5, c));
        }
    }
    let mut x = g.content_x;
    // trailing marks, right to left
    let mut rx = g.marks_right;
    if let Some((mr, mresp)) = &more {
        let open = menu::is_open(mresp);
        if hovered || open {
            let hot = mresp.hovered() || open;
            if hot {
                p.rect_filled(*mr, theme::R_SM, t.hover);
            }
            theme::paint_icon(
                p,
                mr.center(),
                icons::ELLIPSIS,
                theme::ICON_XS,
                if hot { t.text } else { t.text_2 },
            );
        }
    }
    let mut link = None;
    if let Some(label) = row.link {
        let lg = widgets::galley(ui, label, theme::medium(theme::FS_XS), t.accent_text);
        let w = lg.size().x + 12.0;
        let r = Rect::from_min_size(pos2(rx - w, cy - 9.0), Vec2::new(w, 18.0));
        let lr = ui.interact(r, resp.id.with("link"), Sense::click());
        let fill = if lr.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            0.18
        } else {
            0.1
        };
        ui.painter()
            .rect_filled(r, theme::R_XS, theme::alpha(t.accent, fill));
        ui.painter().galley(
            pos2(r.left() + 6.0, cy - lg.size().y / 2.0),
            lg,
            t.accent_text,
        );
        rx = r.left() - 6.0;
        link = Some(lr);
    }
    for a in row.after.iter().rev() {
        rx = paint_after(ui, t, *a, rx, cy) - 6.0;
    }
    if let Some(b) = row.badge {
        let fg = if row.selected { t.text } else { t.text_2 };
        let bg = widgets::galley(ui, b, theme::mono_medium(10.5), fg);
        let w = (bg.size().x + 8.0).max(20.0);
        let r = Rect::from_min_size(pos2(x, cy - 8.0), Vec2::new(w, 16.0));
        ui.painter().rect_filled(
            r,
            3,
            theme::alpha(t.text, if row.selected { 0.12 } else { 0.07 }),
        );
        ui.painter().galley(
            Align2::CENTER_CENTER.anchor_size(r.center(), bg.size()).min,
            bg,
            fg,
        );
        x += w + 6.0;
    }
    let job = label_job(row, t, (rx - x - 4.0).max(0.0));
    let lg = ui.painter().layout_job(job);
    ui.painter()
        .galley(pos2(x, cy - lg.size().y / 2.0), lg, t.text);
    NodeResp {
        row: resp,
        more: more.map(|m| m.1),
        link,
    }
}

/// The label, its filter match in semibold accent text (Rabbit `Highlighted`) and the tag, cut at `room`.
fn label_job(row: &Row<'_>, t: &Theme, room: f32) -> LayoutJob {
    let font = if row.mono {
        theme::mono(theme::FS_DATA)
    } else if row.strong {
        theme::medium(theme::FS)
    } else {
        theme::body()
    };
    let mut job = LayoutJob::default();
    let base = TextFormat::simple(font, t.text);
    match crate::chrome::find_ci(row.label, row.highlight) {
        Some(at) => {
            let end = at + row.highlight.len();
            job.append(&row.label[..at], 0.0, base.clone());
            job.append(
                &row.label[at..end],
                0.0,
                TextFormat::simple(theme::semibold(theme::FS), t.accent_text),
            );
            job.append(&row.label[end..], 0.0, base);
        }
        None => job.append(row.label, 0.0, base),
    }
    if let Some(tag) = row.tag {
        // `.iiot-proto`: `margin-left: 6px`, mono 500 10 px, text-3 (text-2 on a selected row)
        let c = if row.selected { t.text_2 } else { t.text_3 };
        let mut f = TextFormat::simple(theme::mono_medium(10.0), c);
        f.extra_letter_spacing = 0.2;
        job.append(tag, 6.0, f);
    }
    job.wrap = egui::text::TextWrapping::truncate_at_width(room);
    job
}

fn paint_after(ui: &Ui, t: &Theme, a: After<'_>, right: f32, cy: f32) -> f32 {
    let p = ui.painter();
    let text = |s: &str, font: egui::FontId, c: Color32, right: f32| {
        let g = widgets::galley(ui, s, font, c);
        let w = g.size().x;
        p.galley(pos2(right - w, cy - g.size().y / 2.0), g, c);
        right - w
    };
    match a {
        After::Text(s) => text(s, theme::ui_font(theme::FS_XS), t.text_3, right),
        After::Mono(s) => text(s, theme::mono(theme::FS_SM), t.text_3, right),
        After::Hot(icon, s) | After::Warm(icon, s) => {
            let c = if matches!(a, After::Hot(..)) {
                t.danger
            } else {
                t.warn
            };
            let x = text(s, theme::medium(theme::FS_XS), c, right);
            theme::paint_icon(p, pos2(x - 10.0, cy), icon, theme::ICON_XS, c);
            x - 16.0
        }
        After::WarmText(s) => text(s, theme::medium(theme::FS_XS), t.warn, right),
        After::Icon(icon, c) => {
            theme::paint_icon(p, pos2(right - 6.0, cy), icon, theme::ICON_XS, c);
            right - 12.0
        }
        After::Live(c) => {
            widgets::chip::paint_live_dot(p, pos2(right - 4.0, cy), c);
            right - 8.0
        }
        After::Counted(c, s) => {
            let x = text(s, theme::ui_font(theme::FS_XS), t.text_3, right);
            p.rect_filled(
                Rect::from_center_size(pos2(x - 8.0, cy), Vec2::splat(8.0)),
                2,
                c,
            );
            x - 12.0
        }
    }
}

/// The row's menu on right-click and behind its "⋯" (`key` names it for `menu::dev_open`).
pub fn row_menu(nr: &NodeResp, key: impl FnOnce() -> String, mut add: impl FnMut(&mut Ui)) {
    menu::context(&nr.row, &mut add);
    if let Some(more) = &nr.more {
        menu::dev_open(more, key);
        menu::dropdown(more, &mut add);
    }
}

/// The line a filtered tree shows when nothing matches (Rabbit: `px-3 py-2.5 text-sm text-fg-3`), one
/// line: `Nothing matches “{filter}”.`
pub fn nothing_matches(ui: &mut Ui, filter: &str) {
    let t = theme::theme(ui);
    let g = widgets::galley(
        ui,
        format!("Nothing matches \u{201c}{filter}\u{201d}."),
        theme::ui_font(theme::FS_SM),
        t.text_3,
    );
    let (r, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), g.size().y + 20.0),
        Sense::hover(),
    );
    ui.painter()
        .galley(pos2(r.left() + 12.0, r.top() + 10.0), g, t.text_3);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row_rect() -> Rect {
        Rect::from_min_size(pos2(40.0, 100.0), Vec2::new(300.0, 24.0))
    }

    #[test]
    fn rows_indent_sixteen_a_level() {
        let r = row_rect();
        let g0 = geom(r, 0, 14.0, false);
        let g2 = geom(r, 2, 14.0, false);
        assert_eq!(g0.twisty.left(), 46.0);
        assert_eq!(g2.twisty.left() - g0.twisty.left(), 32.0);
        // twisty 16, gap 6, lead 14, gap 6
        assert_eq!(g0.lead.left(), 46.0 + 22.0);
        assert_eq!(g0.content_x, 46.0 + 22.0 + 14.0 + 6.0);
        // the wider dot lead pushes the label 2 px
        assert_eq!(
            geom(r, 0, Lead::Dot(Color32::RED).width(), false).content_x,
            g0.content_x + 2.0
        );
        // both centered on the row
        assert_eq!(g0.twisty.center().y, r.center().y);
        assert_eq!(g0.lead.center().y, r.center().y);
    }

    #[test]
    fn the_more_slot_is_kept_even_while_hidden() {
        let r = row_rect();
        let plain = geom(r, 1, 14.0, false);
        let menu = geom(r, 1, 14.0, true);
        assert_eq!(plain.more, None);
        assert_eq!(plain.marks_right, r.right() - 8.0);
        let m = menu.more.unwrap_or(Rect::NOTHING);
        assert_eq!(m.size(), Vec2::splat(20.0));
        assert_eq!(m.right(), r.right() - 8.0);
        // marks end 6 px before it
        assert_eq!(menu.marks_right, m.left() - 6.0);
    }

    #[test]
    fn hit_testing_finds_the_twisty_the_more_button_and_the_row() {
        let r = row_rect();
        let g = geom(r, 1, 14.0, true);
        assert_eq!(g.hit(r, g.twisty.center()), Part::Twisty);
        assert_eq!(
            g.hit(r, g.more.unwrap_or(Rect::NOTHING).center()),
            Part::More
        );
        assert_eq!(g.hit(r, pos2(g.content_x + 10.0, r.center().y)), Part::Row);
        assert_eq!(g.hit(r, pos2(r.left() - 1.0, r.center().y)), Part::Outside);
        // the "⋯" box is 20 px: one past its left edge is the row again
        let m = g.more.unwrap_or(Rect::NOTHING);
        assert_eq!(g.hit(r, pos2(m.left() - 0.5, m.center().y)), Part::Row);
    }

    #[test]
    fn guides_sit_under_each_parents_twisty() {
        let r = row_rect();
        for level in 0..3 {
            let parent = geom(r, level, 14.0, false);
            // the twisty's centre column, its left pixel
            assert_eq!(guide_x(r.left(), level), parent.twisty.left() + 7.5);
        }
    }
}
