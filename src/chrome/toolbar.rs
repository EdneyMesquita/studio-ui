//! The 40 px main toolbar (Rabbit `Toolbar`, the mockup's `.toolbar`): the app mark, the connection widget,
//! the breadcrumbs, the search box centred on the window, and the app's buttons at the right. The active
//! connection's identity colour tints the leading edge (JetBrains-style project colour). The window keeps
//! the OS's own title bar.
//!
//! ```no_run
//! # use studio_ui::chrome::toolbar::{self, ConnLead, ConnWidget, Crumb, Toolbar};
//! # use studio_ui::{egui, icons, theme::identity, widgets::menu};
//! # fn mark(_: &egui::Painter, _: egui::Rect) {}
//! # fn f(ui: &mut egui::Ui) {
//! let crumbs = [Crumb { icon: icons::GLOBE, label: "plc.example:4840", mono: true }];
//! let tb = Toolbar {
//!     tint: Some(identity::TEAL),
//!     mark: &mark,
//!     conn: ConnWidget { lead: ConnLead::Chip(identity::TEAL, "WT"), label: "Water treatment", strong: true },
//!     crumbs: &crumbs,
//!     show_crumbs: true,
//!     search_hint: "Search devices, nodes, topics, tags",
//!     search_kbd: "Ctrl K",
//! };
//! let out = toolbar::toolbar(ui, &tb, |ui| {
//!     let _ = toolbar::theme_toggle(ui);
//!     let _ = toolbar::record_button(ui, None, "Record this session");
//! });
//! menu::dropdown(&out.conn, |ui| menu::head(ui, "Switch connection"));
//! if out.search.clicked() { /* open the palette */ }
//! # }
//! ```
//!
//! # How the apps map onto it
//!
//! | Mb Studio `chrome/toolbar.rs` | This module |
//! |---|---|
//! | `paint_tint(active connection colour)` and the bottom seam | [`Toolbar::tint`] |
//! | `logo::paint` | [`Toolbar::mark`]` = &logo::paint` |
//! | `conn_widget`: chip + name (medium), `Workspace` with `LAYOUT_GRID`, or `N connected` in text-2 | [`ConnWidget`] with [`ConnLead::Chip`] / [`ConnLead::Icon`]`(icons::LAYOUT_GRID)` / [`ConnLead::None`] and `strong: false` |
//! | its "Switch connection" dropdown and `dev_open("conn-widget")` | `menu::dev_open(&out.conn, ..)`, `menu::dropdown(&out.conn, ..)` |
//! | `crumbs` (serial device or line, `unit N`, the tab), mono when it looks like data | [`Crumb`]s; the app decides `mono` |
//! | crumbs hidden under 960 px (`window >= NARROW`) | `show_crumbs: window >= NARROW` |
//! | the search box (`Search devices, registers, actions`, `Ctrl K`) → `OpenPalette` | [`Toolbar::search_hint`], [`ToolbarResponse::search`] |
//! | right: theme, Connect all, New connection, Record | the `right` closure: [`theme_toggle`], the app's `icon_button`s, [`record_button`] |
//!
//! | OPC Studio `chrome/toolbar.rs` | This module |
//! |---|---|
//! | the server widget ("Switch server") | as Mb's |
//! | crumbs: host:port (`GLOBE`, data), the node or watch list (data), the tab | [`Crumb`] with `mono` from the app's own flag |
//! | crumbs hidden at 960 px and under (`window > NARROW`) | `show_crumbs: window > NARROW` |
//! | right: settings menu, theme, Record | the `right` closure: `icon_button(SETTINGS)` + `menu::dropdown`, [`theme_toggle`], [`record_button`] |
//!
//! The record timer asks for its next repaint with [`crate::chrome::repaint_after`] (OPC's way); Mb asked
//! with a plain `request_repaint_after`, which could wake one frame early.

use egui::{Align, Color32, Layout, Painter, Rect, Response, Sense, Ui, UiBuilder, Vec2, pos2};

use crate::icons;
use crate::theme::{self, Theme};
use crate::widgets::{self, chip::ChipSize, menu};

/// What leads the connection widget.
#[derive(Clone, Copy, Debug)]
pub enum ConnLead<'a> {
    /// The 18 px chip of the active connection.
    Chip(Color32, &'a str),
    /// A 14 px icon in text-2 (the workspace).
    Icon(&'a str),
    /// Nothing (`N connected`).
    None,
}

/// The connection widget (Rabbit `ConnectionWidget`): what the active tab belongs to, and a chevron. The
/// app attaches its switcher with `menu::dropdown(&out.conn, ..)`.
#[derive(Clone, Copy, Debug)]
pub struct ConnWidget<'a> {
    pub lead: ConnLead<'a>,
    pub label: &'a str,
    /// Medium weight in text colour; regular in text-2 otherwise (`N connected`).
    pub strong: bool,
}

/// One breadcrumb: `px-1.5 gap-1.5`, a 14 px icon, the place.
#[derive(Clone, Copy, Debug)]
pub struct Crumb<'a> {
    pub icon: &'a str,
    pub label: &'a str,
    /// The place is data (an address, a path): JetBrains Mono.
    pub mono: bool,
}

/// The toolbar's content; the right-side buttons come as a closure to [`toolbar`].
pub struct Toolbar<'a> {
    /// The active connection's identity colour: tints the leading 360 px.
    pub tint: Option<Color32>,
    /// Paints the app's 18 px mark into the given square.
    pub mark: &'a dyn Fn(&Painter, Rect),
    pub conn: ConnWidget<'a>,
    /// Where the active tab is, after its connection: `/` before each, the last in text colour.
    pub crumbs: &'a [Crumb<'a>],
    /// The app's breakpoint for the crumbs (the siblings hide them in a narrow window).
    pub show_crumbs: bool,
    pub search_hint: &'a str,
    pub search_kbd: &'a str,
}

impl std::fmt::Debug for Toolbar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toolbar")
            .field("tint", &self.tint)
            .field("conn", &self.conn)
            .field("crumbs", &self.crumbs)
            .field("show_crumbs", &self.show_crumbs)
            .field("search_hint", &self.search_hint)
            .field("search_kbd", &self.search_kbd)
            .finish_non_exhaustive()
    }
}

/// The toolbar's clickable parts the app reacts to.
#[derive(Debug)]
pub struct ToolbarResponse {
    pub conn: Response,
    pub search: Response,
}

/// The search box: the middle column of `grid-cols-[1fr_auto_1fr] pr-1.5 pl-2.5`, centered in the padded
/// bar (2 px right of the window's centre), `w-[min(320px,28vw)]`, `CONTROL` high.
pub fn search_rect(bar: Rect, window: f32) -> Rect {
    let sw = (window * 0.28).min(320.0);
    let mid = (bar.left() + 10.0 + bar.right() - 6.0) / 2.0;
    Rect::from_center_size(pos2(mid, bar.center().y), Vec2::new(sw, theme::CONTROL))
}

/// The crumbs' widths in `room`: each crumb's natural width (`natural`, icon and padding included) and the
/// slashes (`slash_w`, never shrunk), `gap-0.5` between all of them. Short of room, the crumbs (`min-w-0
/// truncate`) give way in proportion to their width.
pub fn crumb_widths(natural: &[f32], slash_w: f32, room: f32) -> Vec<f32> {
    let n = natural.len();
    let mut widths = natural.to_vec();
    if n == 0 {
        return widths;
    }
    let total = n as f32 * slash_w + natural.iter().sum::<f32>() + 2.0 * (2 * n - 1) as f32;
    if total > room {
        let deficit = total - room;
        let sum: f32 = natural.iter().sum();
        crate::chrome::shrink_all(&mut widths, sum - deficit);
    }
    widths
}

/// Draw the toolbar into `ui` (a 40 px panel). `right` adds the right-hand buttons, laid out right to left
/// 4 px apart after 6 px of padding (the first one added is the rightmost).
pub fn toolbar(ui: &mut Ui, tb: &Toolbar<'_>, right: impl FnOnce(&mut Ui)) -> ToolbarResponse {
    let t = theme::theme(ui);
    let rect = ui.max_rect();
    paint_tint(ui.painter(), rect, tb.tint, t);
    widgets::hline(ui.painter(), rect.x_range(), rect.bottom() - 0.5, t.seam);

    let window = crate::chrome::window_width(ui);
    let search = search_rect(rect, window);
    let resp = ui.interact(search, ui.id().with("searchbox"), Sense::click());
    let border = if resp.hovered() { t.field_line } else { t.line };
    ui.painter().rect(
        search,
        theme::R,
        t.editor,
        egui::Stroke::new(1.0, border),
        egui::StrokeKind::Inside,
    );
    theme::paint_icon(
        ui.painter(),
        // border, `pl-2.5`, then the 14 px glyph
        pos2(search.left() + 1.0 + 10.0 + 7.0, search.center().y),
        icons::SEARCH,
        theme::ICON_SM,
        t.text_3,
    );
    let kg = widgets::galley(ui, tb.search_kbd, theme::medium(10.5), t.text_2);
    // `pr-1.5` inside the border
    let kr = widgets::kbd_rect_right(&kg, search.right() - 7.0, search.center().y);
    // `gap-2` after the glyph and before the keycap
    let hint_x = search.left() + 1.0 + 10.0 + 14.0 + 8.0;
    let hint = widgets::galley_truncated(
        ui,
        tb.search_hint,
        theme::body(),
        t.text_3,
        kr.left() - 8.0 - hint_x,
    );
    ui.painter().galley(
        pos2(hint_x, search.center().y - hint.size().y / 2.0),
        hint,
        t.text_3,
    );
    widgets::paint_kbd(ui.painter(), kr, kg, t.editor, t.line, t.text_2);
    let search_resp = resp.on_hover_cursor(egui::CursorIcon::Text);

    // left: mark, connection widget, crumbs
    let left = Rect::from_min_max(
        pos2(rect.left() + 10.0, rect.top()),
        pos2(search.left() - 12.0, rect.bottom() - 1.0),
    );
    let mut lui = ui.new_child(
        UiBuilder::new()
            .max_rect(left)
            .layout(Layout::left_to_right(Align::Center)),
    );
    lui.set_clip_rect(left);
    lui.spacing_mut().item_spacing.x = 4.0;
    let (lr, _) = lui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
    (tb.mark)(lui.painter(), lr);
    lui.add_space(6.0);
    let conn = conn_widget(&mut lui, &tb.conn, t);
    if tb.show_crumbs {
        crumbs(&mut lui, tb.crumbs, t, left.right());
    }

    // right: the app's buttons
    let right_rect = Rect::from_min_max(
        pos2(search.right() + 12.0, rect.top()),
        pos2(rect.right(), rect.bottom() - 1.0),
    );
    let mut rui = ui.new_child(
        UiBuilder::new()
            .max_rect(right_rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    rui.spacing_mut().item_spacing.x = 4.0;
    rui.add_space(6.0);
    right(&mut rui);
    ToolbarResponse {
        conn,
        search: search_resp,
    }
}

/// The bar's `panel` fill with the identity tint: 26 % of the colour at the left edge fading to nothing
/// 360 px in.
pub fn paint_tint(p: &Painter, rect: Rect, color: Option<Color32>, t: &Theme) {
    p.rect_filled(rect, 0, t.panel);
    let Some(c) = color else {
        return;
    };
    let lead = theme::mix(c, 0.26, t.panel);
    let x1 = (rect.left() + 360.0).min(rect.right());
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), lead);
    mesh.colored_vertex(pos2(x1, rect.top()), t.panel);
    mesh.colored_vertex(pos2(x1, rect.bottom()), t.panel);
    mesh.colored_vertex(rect.left_bottom(), lead);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(egui::Shape::mesh(mesh));
}

fn conn_widget(ui: &mut Ui, w: &ConnWidget<'_>, t: &Theme) -> Response {
    let (font, color) = if w.strong {
        (theme::medium(theme::FS), t.text)
    } else {
        (theme::body(), t.text_2)
    };
    let g = widgets::galley(ui, w.label, font, color);
    // `px-2 gap-1.5`
    let lead_w = match w.lead {
        ConnLead::Chip(..) => 18.0 + 6.0,
        ConnLead::Icon(_) => theme::ICON_SM + 6.0,
        ConnLead::None => 0.0,
    };
    let width = 8.0 + lead_w + g.size().x + 6.0 + 12.0 + 8.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, theme::CONTROL), Sense::click());
    let open = menu::is_open(&resp);
    if resp.hovered() || open {
        ui.painter().rect_filled(rect, theme::R, t.hover);
    }
    let cy = rect.center().y;
    let x = rect.left() + 8.0;
    match w.lead {
        ConnLead::Chip(c, initials) => widgets::chip::paint_chip(
            ui.painter(),
            Rect::from_min_size(pos2(x, cy - 9.0), Vec2::splat(18.0)),
            c,
            initials,
            ChipSize::Md,
        ),
        ConnLead::Icon(glyph) => theme::paint_icon(
            ui.painter(),
            pos2(x + theme::ICON_SM / 2.0, cy),
            glyph,
            theme::ICON_SM,
            t.text_2,
        ),
        ConnLead::None => {}
    }
    let gw = g.size().x;
    ui.painter()
        .galley(pos2(x + lead_w, cy - g.size().y / 2.0), g, color);
    theme::paint_icon(
        ui.painter(),
        pos2(x + lead_w + gw + 6.0 + 6.0, cy),
        icons::CHEVRON_DOWN,
        theme::ICON_XS,
        t.text_3,
    );
    resp
}

/// The crumbs (Rabbit `Breadcrumb`), up to `right`: `/` then each place, the last in text colour.
fn crumbs(ui: &mut Ui, items: &[Crumb<'_>], t: &Theme, right: f32) {
    if items.is_empty() {
        return;
    }
    // `nav gap-0.5` between the slashes and the crumbs
    ui.spacing_mut().item_spacing.x = 2.0;
    let slash = widgets::galley(ui, "/", theme::body(), t.text_3);
    let font = |mono: bool| if mono { theme::data() } else { theme::body() };
    // a crumb: `px-1.5 gap-1.5`, 14 px icon, the place
    let fixed = 6.0 + theme::ICON_SM + 6.0 + 6.0;
    let natural: Vec<f32> = items
        .iter()
        .map(|c| fixed + widgets::galley(ui, c.label, font(c.mono), t.text).size().x)
        .collect();
    let room = right - ui.cursor().left();
    let widths = crumb_widths(&natural, slash.size().x, room);
    let n = items.len();
    for (k, (c, w)) in items.iter().zip(widths).enumerate() {
        let (r, _) =
            ui.allocate_exact_size(Vec2::new(slash.size().x, theme::CONTROL), Sense::hover());
        ui.painter().galley(
            pos2(r.left(), r.center().y - slash.size().y / 2.0),
            slash.clone(),
            t.text_3,
        );
        let color = if k + 1 == n { t.text } else { t.text_2 };
        let (r, _) = ui.allocate_exact_size(Vec2::new(w, theme::CONTROL), Sense::hover());
        let cy = r.center().y;
        theme::paint_icon(
            ui.painter(),
            pos2(r.left() + 6.0 + theme::ICON_SM / 2.0, cy),
            c.icon,
            theme::ICON_SM,
            color,
        );
        let g = widgets::galley_truncated(ui, c.label, font(c.mono), color, (w - fixed).max(0.0));
        ui.painter().galley(
            pos2(r.left() + 6.0 + theme::ICON_SM + 6.0, cy - g.size().y / 2.0),
            g,
            color,
        );
    }
}

/// The light / dark switch: a sun in the dark theme, a moon in the light one.
pub fn theme_toggle(ui: &mut Ui) -> Response {
    let (glyph, tip) = if theme::theme(ui).dark {
        (icons::SUN, "Switch to light theme")
    } else {
        (icons::MOON, "Switch to dark theme")
    };
    widgets::icon_button(ui, glyph, theme::CONTROL, tip)
}

/// The Record button: `Record` with a ring, or while recording (`elapsed` seconds) a danger-tinted
/// `REC mm:ss` with a dot and a stop square, ticking once a second. `tip` is the idle tooltip (what gets
/// recorded); recording, it is `Stop recording`.
pub fn record_button(ui: &mut Ui, elapsed: Option<f64>, tip: &str) -> Response {
    let t = theme::theme(ui);
    let on = elapsed.is_some();
    let label = match elapsed {
        Some(s) => {
            let s = s.max(0.0) as u64;
            // the timer ticks once a second only while recording
            crate::chrome::repaint_after(ui.ctx(), 1.0 - ui.input(|i| i.time).fract());
            format!("REC {:02}:{:02}", s / 60, s % 60)
        }
        None => "Record".to_owned(),
    };
    let color = if on { t.danger } else { t.text_2 };
    let font = if on {
        theme::mono(theme::FS_SM)
    } else {
        theme::body()
    };
    let g = widgets::galley(ui, label, font, color);
    let w = 16.0 + theme::ICON_SM + 6.0 + g.size().x + if on { 12.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, theme::CONTROL), Sense::click());
    let hot = resp.hovered();
    let bg = match (on, hot) {
        (true, false) => Some(theme::alpha(t.danger, 0.13)),
        (true, true) => Some(theme::alpha(t.danger, 0.2)),
        (false, true) => Some(t.hover),
        _ => None,
    };
    if let Some(bg) = bg {
        ui.painter().rect_filled(rect, theme::R, bg);
    }
    let mut x = rect.left() + 8.0;
    let cy = rect.center().y;
    if on {
        ui.painter().circle_filled(pos2(x + 3.0, cy), 3.0, t.danger);
        x += 12.0;
    }
    let fg = if hot && !on { t.text } else { color };
    theme::paint_icon(
        ui.painter(),
        pos2(x + 7.0, cy),
        if on { icons::SQUARE } else { icons::CIRCLE_DOT },
        theme::ICON_SM,
        fg,
    );
    x += theme::ICON_SM + 6.0;
    ui.painter()
        .galley_with_override_text_color(pos2(x, cy - g.size().y / 2.0), g, fg);
    widgets::tip(resp, if on { "Stop recording" } else { tip })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_search_box_centres_on_the_padded_bar() {
        let bar = Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(1440.0, 40.0));
        let s = search_rect(bar, 1440.0);
        // `w-[min(320px,28vw)]`
        assert_eq!(s.width(), 320.0);
        // pl-2.5 / pr-1.5: 2 px right of the window's centre
        assert_eq!(s.center().x, 722.0);
        assert_eq!(s.height(), theme::CONTROL);
        // a narrow window: 28 % of it
        let s = search_rect(
            Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(800.0, 40.0)),
            800.0,
        );
        assert!((s.width() - 224.0).abs() < 1e-3);
    }

    #[test]
    fn crumbs_clip_in_proportion_and_the_slashes_stay() {
        // room for both: untouched (2 slashes of 5, 3 gaps of 2)
        assert_eq!(crumb_widths(&[100.0, 50.0], 5.0, 200.0), vec![100.0, 50.0]);
        // exactly enough
        assert_eq!(crumb_widths(&[100.0, 50.0], 5.0, 166.0), vec![100.0, 50.0]);
        // 30 short: the crumbs give up 30 between them, 2 : 1
        let w = crumb_widths(&[100.0, 50.0], 5.0, 136.0);
        assert!((w[0] - 80.0).abs() < 1e-3 && (w[1] - 40.0).abs() < 1e-3);
        // no room: both to nothing, never negative
        assert_eq!(crumb_widths(&[100.0, 50.0], 5.0, 0.0), vec![0.0, 0.0]);
        // one crumb, three crumbs
        // one crumb: its slash and one gap stay
        assert_eq!(crumb_widths(&[100.0], 5.0, 57.0), vec![50.0]);
        let w = crumb_widths(&[60.0, 30.0, 10.0], 5.0, 15.0 + 10.0 + 50.0);
        assert!((w.iter().sum::<f32>() - 50.0).abs() < 1e-3);
        assert!(crumb_widths(&[], 5.0, 10.0).is_empty());
    }
}
